use opencv::{
    core::{self, absdiff, Mat, Point, Scalar, Size, BORDER_DEFAULT},
    imgproc::{self, COLOR_BGR2GRAY, THRESH_BINARY},
    prelude::*,
    videoio::{self, VideoCapture},
    Result,
};

pub struct HandTracker {
    cap: VideoCapture,
    prev_gray: Mat,
    prev_cx: Option<f32>,
    prev_cy: Option<f32>,
}

pub struct TrackerData {
    pub delta: Option<(f32, f32)>,
    pub frame_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl HandTracker {
    pub fn new() -> Result<Self> {
        let mut cap = VideoCapture::new(0, videoio::CAP_ANY)?;
        if !cap.is_opened()? {
            return Err(opencv::Error::new(core::StsError, "Cannot open camera"));
        }
        
        let mut first_frame = Mat::default();
        cap.read(&mut first_frame)?;
        
        let mut prev_gray = Mat::default();
        if !first_frame.empty() {
            imgproc::cvt_color(&first_frame, &mut prev_gray, COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
            imgproc::gaussian_blur(&prev_gray.clone(), &mut prev_gray, Size::new(21, 21), 0.0, 0.0, BORDER_DEFAULT, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
        }
        
        Ok(Self {
            cap,
            prev_gray,
            prev_cx: None,
            prev_cy: None,
        })
    }

    pub fn get_delta(&mut self) -> Result<Option<TrackerData>> {
        let mut frame = Mat::default();
        self.cap.read(&mut frame)?;
        if frame.empty() {
            return Ok(None);
        }
        
        // Lật ảnh như gương
        let mut flipped = Mat::default();
        core::flip(&frame, &mut flipped, 1)?;
        frame = flipped;

        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
        imgproc::gaussian_blur(&gray.clone(), &mut gray, Size::new(21, 21), 0.0, 0.0, BORDER_DEFAULT, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;

        let mut diff = Mat::default();
        absdiff(&self.prev_gray, &gray, &mut diff)?;

        let mut thresh = Mat::default();
        imgproc::threshold(&diff, &mut thresh, 25.0, 255.0, THRESH_BINARY)?;
        imgproc::dilate(&thresh.clone(), &mut thresh, &Mat::default(), Point::new(-1, -1), 2, BORDER_DEFAULT, Scalar::default())?;

        let mut contours = core::Vector::<core::Vector<Point>>::new();
        imgproc::find_contours(&thresh, &mut contours, imgproc::RETR_EXTERNAL, imgproc::CHAIN_APPROX_SIMPLE, Point::new(0, 0))?;

        let mut max_area = 0.0;
        let mut best_cnt = None;
        for i in 0..contours.len() {
            let cnt = contours.get(i)?;
            let area = imgproc::contour_area(&cnt, false)?;
            // Lọc nhiễu, tìm vùng chuyển động lớn nhất
            if area > 1000.0 && area > max_area {
                max_area = area;
                best_cnt = Some(cnt);
            }
        }

        self.prev_gray = gray; // Cập nhật frame trước đó
        
        let mut result_delta = None;
        if let Some(cnt) = best_cnt {
            let m = imgproc::moments(&cnt, false)?;
            if m.m00 != 0.0 {
                let cx = (m.m10 / m.m00) as f32;
                let cy = (m.m01 / m.m00) as f32;
                
                if let (Some(px), Some(py)) = (self.prev_cx, self.prev_cy) {
                    let dx = (cx - px) * 2.0; // Hệ số nhạy
                    let dy = (cy - py) * 2.0;
                    result_delta = Some((dx, dy));
                }
                
                self.prev_cx = Some(cx);
                self.prev_cy = Some(cy);
                
                // Vẽ contour để dễ nhìn
                imgproc::draw_contours(&mut frame, &contours, -1, Scalar::new(0.0, 255.0, 0.0, 0.0), 2, imgproc::LINE_8, &core::no_array(), i32::MAX, Point::new(0, 0))?;
            } else {
                self.prev_cx = None;
                self.prev_cy = None;
            }
        } else {
            self.prev_cx = None;
            self.prev_cy = None;
        }
        
        // Convert sang RGBA
        let mut rgba = Mat::default();
        imgproc::cvt_color(&frame, &mut rgba, imgproc::COLOR_BGR2RGBA, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
        let size = rgba.size()?;
        let width = size.width as u32;
        let height = size.height as u32;
        
        let bytes = rgba.data_bytes()?;
        let frame_rgba = bytes.to_vec();
        
        Ok(Some(TrackerData {
            delta: result_delta,
            frame_rgba,
            width,
            height,
        }))
    }
}

