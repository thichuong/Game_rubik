use opencv::{
    Result,
    core::{self},
};
use std::io::{BufReader, Read};
use std::process::{Child, Command, Stdio};

/// A 3D landmark point
#[derive(Clone, Copy, Debug)]
pub struct Landmark3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Upgraded structured hand data parsed from the tracker thread
pub struct TrackerHand {
    /// Handedness: 0 = Left, 1 = Right
    pub handedness: u8,
    /// Gesture: 1 = Whole Hand (rotation), 2 = Index Pointing (interaction)
    pub gesture_type: u8,
    /// Smoothed screen cursor X/Y coordinates
    pub cursor: (f32, f32),
    /// Velocity/swipe delta if active
    pub delta: Option<(f32, f32)>,
    /// 21 3D landmarks mapped relative to camera space
    pub landmarks: Vec<Landmark3d>,
}

/// Data sent from tracker thread to the game
pub struct TrackerData {
    /// List of tracked hands (up to 2)
    pub hands: Vec<TrackerHand>,
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

/// Individual hand state tracking variables (EMA filter + timeout)
#[derive(Default)]
struct HandTrackingState {
    smoothed_cx: Option<f32>,
    smoothed_cy: Option<f32>,
    prev_cx: Option<f32>,
    prev_cy: Option<f32>,
    lost_frames: u32,
}

impl HandTrackingState {
    fn reset(&mut self) {
        self.smoothed_cx = None;
        self.smoothed_cy = None;
        self.prev_cx = None;
        self.prev_cy = None;
    }

    fn handle_lost_frame(&mut self, timeout: u32) {
        self.lost_frames += 1;
        if self.lost_frames >= timeout {
            self.reset();
        }
    }
}

/// Hand tracker using Google MediaPipe Hands via a Python subprocess.
///
/// Pipeline:
/// 1. Spawns Python virtual environment running MediaPipe Hands (max_num_hands=2)
/// 2. Reads structured binary packets from stdout
/// 3. EMA smooths hand cursors and computes swipe deltas separately for left/right hands
/// 4. Decodes 3D landmarks for rendering holographic hands
/// 5. Converts frame to RGBA and returns to Bevy
pub struct HandTracker {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
    config: SkinConfig,
    left_hand: HandTrackingState,
    right_hand: HandTrackingState,
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
            left_hand: HandTrackingState::default(),
            right_hand: HandTrackingState::default(),
        })
    }

    /// Smooth a value using Exponential Moving Average
    fn ema(prev: Option<f32>, raw: f32, alpha: f32) -> f32 {
        match prev {
            Some(p) => p + alpha * (raw - p),
            None => raw,
        }
    }

    /// Process one camera frame and return tracking data.
    pub fn get_delta(&mut self) -> Result<Option<TrackerData>> {
        // Global Header: 21 bytes:
        // 4 bytes: "HAND"
        // 4 bytes: width (u32)
        // 4 bytes: height (u32)
        // 4 bytes: frame_len (u32)
        // 1 byte: hands_count (u8)
        // 4 bytes: reserved (padding)
        let mut global_header = [0u8; 21];
        if let Err(e) = self.reader.read_exact(&mut global_header) {
            return Err(opencv::Error::new(
                core::StsError,
                format!("Failed to read packet header: {e}"),
            ));
        }

        if &global_header[0..4] != b"HAND" {
            return Err(opencv::Error::new(
                core::StsError,
                "Invalid packet header: expected 'HAND'",
            ));
        }

        let w = read_u32_le(&global_header[4..8]);
        let h = read_u32_le(&global_header[8..12]);
        let frame_len = read_u32_le(&global_header[12..16]) as usize;
        let hands_count = global_header[16] as usize;

        // Read hand blocks (268 bytes per hand)
        let mut hands = Vec::with_capacity(hands_count);
        for _ in 0..hands_count {
            let mut hand_block = [0u8; 268];
            if let Err(e) = self.reader.read_exact(&mut hand_block) {
                return Err(opencv::Error::new(
                    core::StsError,
                    format!("Failed to read hand block: {e}"),
                ));
            }

            let handedness = hand_block[0];
            let gesture_type = hand_block[1];
            let cursor_x = read_f32_le(&hand_block[2..6]);
            let cursor_y = read_f32_le(&hand_block[6..10]);

            // Parse 21 landmarks (each is 3 * f32 = 12 bytes, total 252 bytes)
            let mut landmarks = Vec::with_capacity(21);
            for i in 0..21 {
                let start = 10 + i * 12;
                let lx = read_f32_le(&hand_block[start..start + 4]);
                let ly = read_f32_le(&hand_block[start + 4..start + 8]);
                let lz = read_f32_le(&hand_block[start + 8..start + 12]);
                landmarks.push(Landmark3d {
                    x: lx,
                    y: ly,
                    z: lz,
                });
            }

            // Smooth the active cursor position using EMA
            let state = if handedness == 0 {
                &mut self.left_hand
            } else {
                &mut self.right_hand
            };
            state.lost_frames = 0;

            let alpha = self.config.ema_alpha;
            let sx = Self::ema(state.smoothed_cx, cursor_x, alpha);
            let sy = Self::ema(state.smoothed_cy, cursor_y, alpha);
            state.smoothed_cx = Some(sx);
            state.smoothed_cy = Some(sy);

            // Compute delta from previous smoothed position
            let delta = if let (Some(px), Some(py)) = (state.prev_cx, state.prev_cy) {
                let dx = sx - px;
                let dy = sy - py;

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

            state.prev_cx = Some(sx);
            state.prev_cy = Some(sy);

            hands.push(TrackerHand {
                handedness,
                gesture_type,
                cursor: (sx, sy),
                delta,
                landmarks,
            });
        }

        // Handle lost frames for hands that were not detected in this packet
        let mut left_detected = false;
        let mut right_detected = false;
        for hand in &hands {
            if hand.handedness == 0 {
                left_detected = true;
            } else {
                right_detected = true;
            }
        }

        if !left_detected {
            self.left_hand.handle_lost_frame(self.config.lost_timeout);
        }
        if !right_detected {
            self.right_hand.handle_lost_frame(self.config.lost_timeout);
        }

        // Read the raw frame bytes
        let mut frame_rgba = vec![0u8; frame_len];
        if let Err(e) = self.reader.read_exact(&mut frame_rgba) {
            return Err(opencv::Error::new(
                core::StsError,
                format!("Failed to read frame bytes: {e}"),
            ));
        }

        Ok(Some(TrackerData {
            hands,
            frame_rgba,
            width: w,
            height: h,
        }))
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
