use opencv::{
    Result,
    core::{self, BORDER_DEFAULT, Mat, Point, Scalar, Size, Vector},
    imgproc::{self, COLOR_BGR2HSV, MORPH_ELLIPSE},
    prelude::*,
    videoio::{self, VideoCapture},
};

/// Data sent from tracker thread to the game
pub struct TrackerData {
    /// Rotation delta if the hand is actively swiping
    pub delta: Option<(f32, f32)>,
    /// Smoothed hand center position (if detected)
    pub hand_center: Option<(f32, f32)>,
    /// Camera frame in RGBA format for UI display
    pub frame_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Configuration for skin detection and swipe tracking
pub struct SkinConfig {
    pub h_min: f64,
    pub h_max: f64,
    pub s_min: f64,
    pub s_max: f64,
    pub v_min: f64,
    pub v_max: f64,
    /// Minimum contour area to be considered a hand
    pub min_contour_area: f64,
    /// Sensitivity multiplier for rotation delta
    pub sensitivity: f32,
    /// EMA alpha for centroid smoothing (lower = smoother, higher = more responsive)
    pub ema_alpha: f32,
    /// Dead zone: deltas with magnitude below this (in pixels) are ignored
    pub dead_zone: f32,
    /// Number of consecutive lost frames before resetting tracking
    pub lost_timeout: u32,
}

impl Default for SkinConfig {
    fn default() -> Self {
        Self {
            h_min: 0.0,
            h_max: 25.0,
            s_min: 40.0,
            s_max: 170.0,
            v_min: 60.0,
            v_max: 255.0,
            min_contour_area: 5000.0,
            sensitivity: 2.0,
            ema_alpha: 0.4,
            dead_zone: 3.0,
            lost_timeout: 5,
        }
    }
}

/// Simple swipe-based hand tracker using skin color segmentation.
///
/// Pipeline:
/// 1. HSV skin segmentation → morphological cleanup
/// 2. Find largest skin contour → compute centroid
/// 3. Smooth centroid with EMA → compute frame-to-frame delta
/// 4. Apply dead zone filter → emit delta as rotation input
///
/// No complex gesture recognition needed — just track hand movement
/// like a finger swiping on a touchscreen.
pub struct HandTracker {
    cap: VideoCapture,
    config: SkinConfig,
    // EMA-smoothed centroid position
    smoothed_cx: Option<f32>,
    smoothed_cy: Option<f32>,
    // Previous smoothed position for delta calculation
    prev_cx: Option<f32>,
    prev_cy: Option<f32>,
    // Counter for consecutive frames without hand detection
    lost_frames: u32,
    // Pre-allocated Mat buffers
    buf_hsv: Mat,
    buf_mask: Mat,
    buf_morph: Mat,
    buf_small: Mat,
    buf_rgba: Mat,
}

impl HandTracker {
    /// Create a new hand tracker, opening the default camera
    pub fn new() -> Result<Self> {
        let mut cap = VideoCapture::new(0, videoio::CAP_ANY)?;
        if !cap.is_opened()? {
            return Err(opencv::Error::new(core::StsError, "Cannot open camera"));
        }

        // Read one frame to warm up the camera
        let mut warmup = Mat::default();
        cap.read(&mut warmup)?;

        Ok(Self {
            cap,
            config: SkinConfig::default(),
            smoothed_cx: None,
            smoothed_cy: None,
            prev_cx: None,
            prev_cy: None,
            lost_frames: 0,
            buf_hsv: Mat::default(),
            buf_mask: Mat::default(),
            buf_morph: Mat::default(),
            buf_small: Mat::default(),
            buf_rgba: Mat::default(),
        })
    }

    /// Smooth a value using Exponential Moving Average
    fn ema(prev: Option<f32>, raw: f32, alpha: f32) -> f32 {
        match prev {
            Some(p) => p + alpha * (raw - p),
            None => raw,
        }
    }

    /// Reset all tracking state (called when hand is lost for too long)
    fn reset_tracking(&mut self) {
        self.smoothed_cx = None;
        self.smoothed_cy = None;
        self.prev_cx = None;
        self.prev_cy = None;
    }

    /// Process one camera frame and return tracking data.
    pub fn get_delta(&mut self) -> Result<Option<TrackerData>> {
        let mut frame = Mat::default();
        self.cap.read(&mut frame)?;
        if frame.empty() {
            return Ok(None);
        }

        // Mirror the frame horizontally (webcam mirror effect)
        let mut flipped = Mat::default();
        core::flip(&frame, &mut flipped, 1)?;
        frame = flipped;

        // Resize for gesture processing (keep original for display)
        imgproc::resize(
            &frame,
            &mut self.buf_small,
            Size::new(320, 240),
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )?;

        // --- Skin Color Segmentation ---
        imgproc::cvt_color(
            &self.buf_small,
            &mut self.buf_hsv,
            COLOR_BGR2HSV,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let lower = Scalar::new(self.config.h_min, self.config.s_min, self.config.v_min, 0.0);
        let upper = Scalar::new(self.config.h_max, self.config.s_max, self.config.v_max, 0.0);
        core::in_range(&self.buf_hsv, &lower, &upper, &mut self.buf_mask)?;

        // Morphological cleanup: erode → dilate → median blur
        let kernel_small =
            imgproc::get_structuring_element(MORPH_ELLIPSE, Size::new(3, 3), Point::new(-1, -1))?;
        let kernel_large =
            imgproc::get_structuring_element(MORPH_ELLIPSE, Size::new(5, 5), Point::new(-1, -1))?;

        // erode: mask → morph
        imgproc::erode(
            &self.buf_mask,
            &mut self.buf_morph,
            &kernel_small,
            Point::new(-1, -1),
            2,
            BORDER_DEFAULT,
            Scalar::default(),
        )?;
        // dilate: morph → mask
        imgproc::dilate(
            &self.buf_morph,
            &mut self.buf_mask,
            &kernel_large,
            Point::new(-1, -1),
            3,
            BORDER_DEFAULT,
            Scalar::default(),
        )?;
        // median blur: mask → morph (final skin mask)
        imgproc::median_blur(&self.buf_mask, &mut self.buf_morph, 7)?;

        // --- Find Contours ---
        let mut contours = Vector::<Vector<Point>>::new();
        imgproc::find_contours(
            &self.buf_morph,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        // Find the largest contour above area threshold
        let mut max_area = 0.0;
        let mut best_idx: Option<usize> = None;
        for i in 0..contours.len() {
            if let Ok(cnt) = contours.get(i)
                && let Ok(area) = imgproc::contour_area(&cnt, false)
                && area > self.config.min_contour_area
                && area > max_area
            {
                max_area = area;
                best_idx = Some(i);
            }
        }

        // Scale factors for mapping small-frame coords back to display frame
        let frame_size = frame.size()?;
        let scale_x = frame_size.width as f32 / 320.0;
        let scale_y = frame_size.height as f32 / 240.0;

        // --- Compute centroid and delta ---
        let (hand_center, delta) = if let Some(idx) = best_idx {
            if let Ok(cnt) = contours.get(idx) {
                // Draw contour on display frame
                self.draw_contour_scaled(&mut frame, &cnt, scale_x, scale_y);

                // Compute centroid on small frame
                let m = imgproc::moments(&cnt, false)?;
                if m.m00 > 0.0 {
                    let raw_cx = (m.m10 / m.m00) as f32 * scale_x;
                    let raw_cy = (m.m01 / m.m00) as f32 * scale_y;

                    self.lost_frames = 0;

                    // EMA smooth the centroid
                    let alpha = self.config.ema_alpha;
                    let sx = Self::ema(self.smoothed_cx, raw_cx, alpha);
                    let sy = Self::ema(self.smoothed_cy, raw_cy, alpha);
                    self.smoothed_cx = Some(sx);
                    self.smoothed_cy = Some(sy);

                    // Compute delta from previous smoothed position
                    let delta = if let (Some(px), Some(py)) = (self.prev_cx, self.prev_cy) {
                        let dx = sx - px;
                        let dy = sy - py;

                        // Dead zone: ignore micro-movements
                        let mag_sq = dx * dx + dy * dy;
                        let dz_sq = self.config.dead_zone * self.config.dead_zone;
                        if mag_sq > dz_sq {
                            let sens = self.config.sensitivity;
                            Some((dx * sens, dy * sens))
                        } else {
                            None
                        }
                    } else {
                        // First frame with detection — no delta yet
                        None
                    };

                    self.prev_cx = Some(sx);
                    self.prev_cy = Some(sy);

                    (Some((sx, sy)), delta)
                } else {
                    self.handle_lost_frame();
                    (None, None)
                }
            } else {
                self.handle_lost_frame();
                (None, None)
            }
        } else {
            self.handle_lost_frame();
            (None, None)
        };

        // --- Draw debug overlay ---
        let status = if delta.is_some() {
            "SWIPING"
        } else if hand_center.is_some() {
            "TRACKING"
        } else {
            "NO HAND"
        };
        let _ = imgproc::put_text(
            &mut frame,
            status,
            Point::new(10, 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            1.0,
            Scalar::new(0.0, 255.0, 255.0, 0.0),
            2,
            imgproc::LINE_AA,
            false,
        );

        if let Some((cx, cy)) = hand_center {
            let _ = imgproc::circle(
                &mut frame,
                Point::new(cx as i32, cy as i32),
                8,
                Scalar::new(255.0, 0.0, 255.0, 0.0),
                -1,
                imgproc::LINE_AA,
                0,
            );
        }

        // --- Convert to RGBA for Bevy display ---
        imgproc::cvt_color(
            &frame,
            &mut self.buf_rgba,
            imgproc::COLOR_BGR2RGBA,
            0,
            core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        let size = self.buf_rgba.size()?;
        let width = size.width as u32;
        let height = size.height as u32;
        let bytes = self.buf_rgba.data_bytes()?;
        let frame_rgba = bytes.to_vec();

        Ok(Some(TrackerData {
            delta,
            hand_center,
            frame_rgba,
            width,
            height,
        }))
    }

    /// Handle a frame where no hand was detected
    fn handle_lost_frame(&mut self) {
        self.lost_frames += 1;
        if self.lost_frames >= self.config.lost_timeout {
            self.reset_tracking();
        }
    }

    /// Draw a contour scaled from small-frame coordinates to display-frame coordinates
    fn draw_contour_scaled(
        &self,
        frame: &mut Mat,
        contour: &Vector<Point>,
        scale_x: f32,
        scale_y: f32,
    ) {
        let mut scaled_points = Vector::<Point>::new();
        for j in 0..contour.len() {
            if let Ok(p) = contour.get(j) {
                scaled_points.push(Point::new(
                    (p.x as f32 * scale_x) as i32,
                    (p.y as f32 * scale_y) as i32,
                ));
            }
        }
        let mut scaled_contours = Vector::<Vector<Point>>::new();
        scaled_contours.push(scaled_points);
        let _ = imgproc::draw_contours(
            frame,
            &scaled_contours,
            0,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            &core::no_array(),
            i32::MAX,
            Point::new(0, 0),
        );
    }
}
