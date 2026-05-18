use opencv::{
    Result,
    core::{self},
};
use std::io::{BufReader, Read};
use std::process::{Child, Command, Stdio};

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

/// Hand tracker using Google ML Kit / MediaPipe via a Python subprocess.
///
/// Pipeline:
/// 1. Spawns Python virtual environment running MediaPipe Hands
/// 2. Reads structured binary packets from stdout
/// 3. EMA smooths hand center and computes swipe delta
/// 4. Converts frame to RGBA and returns to Bevy
pub struct HandTracker {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
    config: SkinConfig,
    smoothed_cx: Option<f32>,
    smoothed_cy: Option<f32>,
    prev_cx: Option<f32>,
    prev_cy: Option<f32>,
    lost_frames: u32,
}

impl HandTracker {
    /// Create a new hand tracker, spawning the Python MediaPipe worker
    pub fn new() -> Result<Self> {
        let python_path = "hand_tracker/.venv/bin/python";
        let script_path = "hand_tracker/hand_tracker.py";

        let mut child = Command::new(python_path)
            .arg(script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                opencv::Error::new(
                    core::StsError,
                    format!("Failed to spawn Python MediaPipe process: {e}"),
                )
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            opencv::Error::new(core::StsError, "Failed to capture Python stdout stream")
        })?;
        let reader = BufReader::new(stdout);

        Ok(Self {
            child,
            reader,
            config: SkinConfig::default(),
            smoothed_cx: None,
            smoothed_cy: None,
            prev_cx: None,
            prev_cy: None,
            lost_frames: 0,
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
        // Header is 25 bytes:
        // 4 bytes: "HAND"
        // 4 bytes: width (u32)
        // 4 bytes: height (u32)
        // 1 byte: has_center (u8)
        // 4 bytes: cx (f32)
        // 4 bytes: cy (f32)
        // 4 bytes: frame_len (u32)
        let mut header = [0u8; 25];
        if let Err(e) = self.reader.read_exact(&mut header) {
            return Err(opencv::Error::new(
                core::StsError,
                format!("Failed to read packet header: {e}"),
            ));
        }

        if &header[0..4] != b"HAND" {
            return Err(opencv::Error::new(
                core::StsError,
                "Invalid packet header: expected 'HAND'",
            ));
        }

        let w = read_u32_le(&header[4..8]);
        let h = read_u32_le(&header[8..12]);
        let has_center = header[12] != 0;
        let cx = read_f32_le(&header[13..17]);
        let cy = read_f32_le(&header[17..21]);
        let frame_len = read_u32_le(&header[21..25]) as usize;

        // Read the raw frame bytes
        let mut frame_rgba = vec![0u8; frame_len];
        if let Err(e) = self.reader.read_exact(&mut frame_rgba) {
            return Err(opencv::Error::new(
                core::StsError,
                format!("Failed to read frame bytes: {e}"),
            ));
        }

        // Compute centroid and delta
        let (hand_center, delta) = if has_center {
            self.lost_frames = 0;

            // EMA smooth the centroid
            let alpha = self.config.ema_alpha;
            let sx = Self::ema(self.smoothed_cx, cx, alpha);
            let sy = Self::ema(self.smoothed_cy, cy, alpha);
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
                None
            };

            self.prev_cx = Some(sx);
            self.prev_cy = Some(sy);

            (Some((sx, sy)), delta)
        } else {
            self.handle_lost_frame();
            (None, None)
        };

        Ok(Some(TrackerData {
            delta,
            hand_center,
            frame_rgba,
            width: w,
            height: h,
        }))
    }

    /// Handle a frame where no hand was detected
    fn handle_lost_frame(&mut self) {
        self.lost_frames += 1;
        if self.lost_frames >= self.config.lost_timeout {
            self.reset_tracking();
        }
    }
}

impl Drop for HandTracker {
    fn drop(&mut self) {
        // Terminate the worker subprocess on drop
        let _ = self.child.kill();
    }
}

/// Read a u32 from a little-endian byte slice safely without panics
fn read_u32_le(slice: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    if slice.len() >= 4 {
        bytes.copy_from_slice(&slice[..4]);
    }
    u32::from_le_bytes(bytes)
}

/// Read a f32 from a little-endian byte slice safely without panics
fn read_f32_le(slice: &[u8]) -> f32 {
    let mut bytes = [0u8; 4];
    if slice.len() >= 4 {
        bytes.copy_from_slice(&slice[..4]);
    }
    f32::from_le_bytes(bytes)
}
