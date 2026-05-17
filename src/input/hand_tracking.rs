use crate::events::{CameraFrameEvent, HandRotationEvent};
use bevy::prelude::*;
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::thread;

pub struct HandTrackingPlugin;

#[derive(Resource)]
pub struct HandTrackingEnabled(pub bool);

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HandTrackingEnabled(false))
            .add_message::<HandRotationEvent>()
            .add_message::<CameraFrameEvent>()
            .add_systems(Startup, setup_camera_listener)
            .add_systems(Update, receive_hand_tracking);
    }
}

#[derive(Resource)]
struct HandTrackingReceiver(Mutex<Receiver<hand_tracker::TrackerData>>);

fn setup_camera_listener(mut commands: Commands) {
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let mut tracker = match hand_tracker::HandTracker::new() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to initialize camera tracker: {e}");
                return;
            }
        };
        
        loop {
            if let Ok(Some(data)) = tracker.get_delta() {
                let _ = tx.send(data);
            }
            // Sleep short amount to not hog CPU 100%
            thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    commands.insert_resource(HandTrackingReceiver(Mutex::new(rx)));
}

fn receive_hand_tracking(
    receiver: Option<Res<HandTrackingReceiver>>,
    enabled: Res<HandTrackingEnabled>,
    mut rot_events: MessageWriter<HandRotationEvent>,
    mut frame_events: MessageWriter<CameraFrameEvent>,
) {
    if let Some(receiver) = receiver {
        if let Ok(rx) = receiver.0.lock() {
            for data in rx.try_iter() {
                // Always send the frame to the UI if we receive one
                frame_events.write(CameraFrameEvent {
                    frame_rgba: data.frame_rgba,
                    width: data.width,
                    height: data.height,
                });
                
                // Only send rotation events if the tracking is enabled
                if enabled.0 {
                    if let Some((dx, dy)) = data.delta {
                        rot_events.write(HandRotationEvent {
                            delta_x: dx,
                            delta_y: dy,
                        });
                    }
                }
            }
        }
    }
}
