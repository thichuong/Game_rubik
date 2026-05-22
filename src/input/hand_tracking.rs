use crate::events::{CameraFrameEvent, HandRotationEvent};
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{RotationQueue, RubikSize};
use crate::rubik::systems::creation::GAP;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rubik_solver::StepByStepSolution;

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct HandTrackingPlugin;

#[derive(Resource)]
pub struct HandHoverMaterials {
    pub white: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
    pub orange: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub blue: Handle<StandardMaterial>,
}

#[derive(Component)]
struct HandHovered;

#[derive(Resource)]
pub struct HandTrackerProcess(pub Arc<Mutex<Option<std::process::Child>>>);

impl Drop for HandTrackerProcess {
    fn drop(&mut self) {
        // Kill the Python subprocess on resource drop (app exit)
        if let Ok(mut guard) = self.0.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
}

#[derive(Resource)]
pub struct HandTrackingEnabled(pub bool);

#[derive(Default, Clone, Copy)]
pub struct SingleHandDragState {
    pub start_face: Option<(Entity, Vec3, Vec3)>, // (Entity, normal, hit_point)
    pub prev_gesture_type: u8,
}

#[derive(Resource, Default)]
pub struct HandDragState {
    pub left: SingleHandDragState,
    pub right: SingleHandDragState,
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HandTrackingEnabled(false))
            .insert_resource(HandDragState::default())
            .add_message::<HandRotationEvent>()
            .add_message::<CameraFrameEvent>()
            .add_systems(Startup, (setup_camera_listener, setup_hand_hover_materials))
            .add_systems(Update, (receive_hand_tracking, update_hand_hover));
    }
}

#[derive(Resource)]
struct HandTrackingReceiver(Mutex<Receiver<hand_tracker::TrackerData>>);

fn setup_camera_listener(mut commands: Commands) {
    let (tx, rx) = mpsc::channel();

    let (mut tracker, shared_child) = match hand_tracker::HandTracker::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize camera tracker: {e}");
            return;
        }
    };

    commands.insert_resource(HandTrackerProcess(shared_child));

    thread::spawn(move || {
        loop {
            match tracker.get_delta() {
                Ok(Some(data)) => {
                    let _ = tx.send(data);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Hand tracker error: {e}");
                    // Break the loop if the pipe is broken (process killed or exited)
                    break;
                }
            }
            // No sleep here: block naturally on pipe read_exact to ensure absolute real-time synchronization
        }
    });

    commands.insert_resource(HandTrackingReceiver(Mutex::new(rx)));
}

#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::too_many_lines
)]
fn receive_hand_tracking(
    receiver: Option<Res<HandTrackingReceiver>>,
    enabled: Res<HandTrackingEnabled>,
    mut drag_state: ResMut<HandDragState>,
    mut rotation_queue: ResMut<RotationQueue>,
    mut rot_events: MessageWriter<HandRotationEvent>,
    mut frame_events: MessageWriter<CameraFrameEvent>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    cubie_faces: Query<(Entity, &CubieFace, &GlobalTransform)>,
    cube_query: Single<&GlobalTransform, With<RubikCube>>,
    rubik_size: Res<RubikSize>,
    mut solution: ResMut<StepByStepSolution>,
) {
    let Some(receiver) = receiver else {
        return;
    };
    let Ok(rx) = receiver.0.lock() else {
        return;
    };

    let window = *window_query;
    let (camera, camera_transform) = *camera_query;
    let cube_transform = *cube_query;

    // Drain the channel and only keep the latest packet to eliminate accumulated latency (I/O lag)
    let mut latest_data = None;
    for data in rx.try_iter() {
        latest_data = Some(data);
    }

    let Some(data) = latest_data else {
        return;
    };

    // Send camera frame to UI
    frame_events.write(CameraFrameEvent {
        frame_rgba: data.frame_rgba,
        width: data.width,
        height: data.height,
    });

    let hands = &data.hands;

    if enabled.0
        && !solution.is_searching
        && !(solution.active && solution.current_step < solution.moves.len())
    {
        let mut left_active = false;
        let mut right_active = false;

        for hand in hands {
            if hand.handedness == 0 {
                left_active = true;
            } else {
                right_active = true;
            }

            let drag_sub_state = if hand.handedness == 0 {
                &mut drag_state.left
            } else {
                &mut drag_state.right
            };

            if hand.gesture_type == 1 {
                // Open hand: Rotate the entire Rubik's cube
                if drag_sub_state.prev_gesture_type == 2 || drag_sub_state.prev_gesture_type == 3 {
                    drag_sub_state.start_face = None;
                }
                if let Some((dx, dy)) = hand.delta {
                    rot_events.write(HandRotationEvent {
                        delta_x: dx,
                        delta_y: dy,
                    });
                }
                drag_sub_state.prev_gesture_type = 1;
            } else if hand.gesture_type == 2 {
                // Reset start face when transitioning from folded (Gesture 3) to extended (Gesture 2)
                if drag_sub_state.prev_gesture_type == 3 {
                    drag_sub_state.start_face = None;
                }
                // Index Extended: Hover to select the start face under finger
                process_face_hover_select(
                    hand.cursor,
                    data.width,
                    data.height,
                    drag_sub_state,
                    window,
                    camera,
                    camera_transform,
                    &cubie_faces,
                );
                drag_sub_state.prev_gesture_type = 2;
            } else if hand.gesture_type == 3 {
                // Index Folded: Swipe and rotate the face in the fold direction
                // Support continuous fluid dragging while index finger remains folded
                if (drag_sub_state.prev_gesture_type == 2 || drag_sub_state.prev_gesture_type == 3)
                    && drag_sub_state.start_face.is_some()
                {
                    if solution.active {
                        solution.active = false;
                    }
                    execute_face_swipe_rotation(
                        hand.cursor,
                        data.width,
                        data.height,
                        drag_sub_state,
                        &mut rotation_queue,
                        window,
                        camera,
                        camera_transform,
                        &cubie_faces,
                        cube_transform,
                        *rubik_size,
                    );
                }
                drag_sub_state.prev_gesture_type = 3;
            } else {
                // Clean up face drag for other/unknown/idle gestures
                if drag_sub_state.prev_gesture_type == 2 || drag_sub_state.prev_gesture_type == 3 {
                    drag_sub_state.start_face = None;
                }
                drag_sub_state.prev_gesture_type = hand.gesture_type;
            }
        }

        // Clean up drag state for hands that disappeared in this frame
        if !left_active
            && (drag_state.left.prev_gesture_type == 2 || drag_state.left.prev_gesture_type == 3)
        {
            drag_state.left.start_face = None;
            drag_state.left.prev_gesture_type = 0;
        }
        if !right_active
            && (drag_state.right.prev_gesture_type == 2 || drag_state.right.prev_gesture_type == 3)
        {
            drag_state.right.start_face = None;
            drag_state.right.prev_gesture_type = 0;
        }
    }
}

/// Detect and continuously select the cubie face under the extended index finger
#[allow(clippy::cast_precision_loss)]
fn process_face_hover_select(
    cursor: (f32, f32),
    tracker_w: u32,
    tracker_h: u32,
    drag_state: &mut SingleHandDragState,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cubie_faces: &Query<(Entity, &CubieFace, &GlobalTransform)>,
) {
    let window_width = window.width();
    let window_height = window.height();
    let mapped_x = (cursor.0 / tracker_w as f32) * window_width;
    let mapped_y = (cursor.1 / tracker_h as f32) * window_height;
    let mapped_cursor = Vec2::new(mapped_x, mapped_y);

    let Ok(ray) = camera.viewport_to_world(camera_transform, mapped_cursor) else {
        return;
    };

    let mut closest_hit = None;
    let mut min_dist = f32::MAX;

    for (entity, _cubie_face, transform) in cubie_faces.iter() {
        let normal = transform.back();
        let center = transform.translation();

        let denom = ray.direction.dot(*normal);
        if denom.abs() > 1e-6 {
            let t = (center.dot(*normal) - ray.origin.dot(*normal)) / denom;
            if t > 0.0 && t < min_dist {
                let hit_point = ray.origin + *ray.direction * t;
                let local_hit = hit_point - center;
                let right = transform.right();
                let up = transform.up();

                if local_hit.dot(*right).abs() <= 0.51 && local_hit.dot(*up).abs() <= 0.51 {
                    min_dist = t;
                    closest_hit = Some((entity, *normal, hit_point));
                }
            }
        }
    }

    if let Some((entity, normal, hit_point)) = closest_hit {
        drag_state.start_face = Some((entity, normal, hit_point));
    }
}

/// Execute continuous face rotation based on the finger fold drag direction
#[allow(
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn execute_face_swipe_rotation(
    cursor: (f32, f32),
    tracker_w: u32,
    tracker_h: u32,
    drag_state: &mut SingleHandDragState,
    rotation_queue: &mut RotationQueue,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cubie_faces: &Query<(Entity, &CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    rubik_size: RubikSize,
) {
    // Read the start face, but do NOT remove it to support continuous fluid dragging
    let Some((start_entity, start_normal, start_hit_point)) = drag_state.start_face else {
        return;
    };

    let window_width = window.width();
    let window_height = window.height();
    let mapped_x = (cursor.0 / tracker_w as f32) * window_width;
    let mapped_y = (cursor.1 / tracker_h as f32) * window_height;
    let mapped_cursor = Vec2::new(mapped_x, mapped_y);

    let Ok(ray) = camera.viewport_to_world(camera_transform, mapped_cursor) else {
        return;
    };

    let denom = ray.direction.dot(start_normal);
    if denom.abs() > 1e-6 {
        let t = (start_hit_point.dot(start_normal) - ray.origin.dot(start_normal)) / denom;
        let end_hit_point = ray.origin + *ray.direction * t;
        let drag_vec = end_hit_point - start_hit_point;

        // Perform drag rotation if movement threshold is exceeded (0.55 is less sensitive and safer)
        if drag_vec.length() > 0.55 {
            let drag_dir = drag_vec.normalize();
            let rotation_axis_vec = start_normal.cross(drag_dir);

            let mut best_axis = RotationAxis::X;
            let mut max_dot = 0.0;
            let mut best_local_axis_in_world = Vec3::X;

            for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
                let local_axis_in_world = cube_transform.affine().transform_vector3(axis.vector());
                let dot = rotation_axis_vec.dot(local_axis_in_world).abs();
                if dot > max_dot {
                    max_dot = dot;
                    best_axis = axis;
                    best_local_axis_in_world = local_axis_in_world;
                }
            }

            let sign = rotation_axis_vec.dot(best_local_axis_in_world).signum();
            let direction = if sign > 0.0 {
                Direction::CounterClockwise
            } else {
                Direction::Clockwise
            };

            let index = {
                if let Ok((_ent, _face, transform)) = cubie_faces.get(start_entity) {
                    let cubie_pos = transform.translation();
                    let local_pos = cube_transform
                        .affine()
                        .inverse()
                        .transform_point3(cubie_pos);
                    let size = rubik_size.size;
                    let scale = 3.0 / size as f32;
                    let current_gap = GAP * scale;
                    let offset = (size as f32 - 1.0) / 2.0;
                    match best_axis {
                        RotationAxis::X => ((local_pos.x / current_gap) + offset).round() as i32,
                        RotationAxis::Y => ((local_pos.y / current_gap) + offset).round() as i32,
                        RotationAxis::Z => ((local_pos.z / current_gap) + offset).round() as i32,
                    }
                } else {
                    0
                }
            };

            let size = rubik_size.size;
            if size % 2 == 0 || index != size / 2 {
                rotation_queue.0.push_back(RotationMove {
                    axis: best_axis,
                    index,
                    direction,
                    add_to_history: true,
                });
            }

            // Fluid dragging: Update the start hit point to current hit point so the user can drag continuously
            drag_state.start_face = Some((start_entity, start_normal, end_hit_point));
        }
    }
}

fn setup_hand_hover_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hover_materials = HandHoverMaterials {
        white: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::WHITE),
            emissive: LinearRgba::new(5.0, 5.0, 5.0, 1.0),
            ..default()
        }),
        yellow: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.9, 0.0, 1.0)),
            emissive: LinearRgba::new(6.0, 5.4, 0.0, 1.0),
            ..default()
        }),
        red: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.9, 0.1, 0.1, 1.0)),
            emissive: LinearRgba::new(9.0, 1.0, 1.0, 1.0),
            ..default()
        }),
        orange: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(1.0, 0.4, 0.0, 1.0)),
            emissive: LinearRgba::new(9.0, 3.6, 0.0, 1.0),
            ..default()
        }),
        green: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.7, 0.1, 1.0)),
            emissive: LinearRgba::new(1.0, 7.0, 1.0, 1.0),
            ..default()
        }),
        blue: materials.add(StandardMaterial {
            base_color: Color::Srgba(Srgba::new(0.1, 0.2, 0.9, 1.0)),
            emissive: LinearRgba::new(1.0, 2.0, 9.0, 1.0),
            ..default()
        }),
    };
    commands.insert_resource(hover_materials);
}

/// Highlight ONLY the single cubie face that is currently selected in `HandDragState`
#[allow(clippy::type_complexity)]
fn update_hand_hover(
    mut commands: Commands,
    enabled: Res<HandTrackingEnabled>,
    drag_state: Res<HandDragState>,
    mut face_material_query: Query<(
        Entity,
        &CubieFace,
        &mut MeshMaterial3d<StandardMaterial>,
        Option<&HandHovered>,
    )>,
    rubik_materials: Res<crate::rubik::resources::RubikMaterials>,
    hover_materials: Res<HandHoverMaterials>,
) {
    // 1. Restore previously hovered cubie faces to their original solid materials
    for (entity, face, mut material, hovered) in &mut face_material_query {
        if hovered.is_some() {
            let original_mat = match face.0 {
                crate::rubik::components::Face::Up => &rubik_materials.white,
                crate::rubik::components::Face::Down => &rubik_materials.yellow,
                crate::rubik::components::Face::Left => &rubik_materials.orange,
                crate::rubik::components::Face::Right => &rubik_materials.red,
                crate::rubik::components::Face::Front => &rubik_materials.green,
                crate::rubik::components::Face::Back => &rubik_materials.blue,
            };
            *material = MeshMaterial3d(original_mat.clone());
            commands.entity(entity).remove::<HandHovered>();
        }
    }

    // 2. If hand tracking is disabled, stop here
    if !enabled.0 {
        return;
    }

    // 3. Collect active selected start faces from drag state (maximum 1 face highlighted per hand)
    let mut faces_to_hover = Vec::new();
    if let Some((entity, _, _)) = drag_state.left.start_face {
        faces_to_hover.push(entity);
    }
    if let Some((entity, _, _)) = drag_state.right.start_face {
        faces_to_hover.push(entity);
    }

    // 4. Apply glowing hover materials to these specific active faces
    for entity in faces_to_hover {
        if let Ok((_ent, face, mut material, _hovered)) = face_material_query.get_mut(entity) {
            let hover_mat = match face.0 {
                crate::rubik::components::Face::Up => &hover_materials.white,
                crate::rubik::components::Face::Down => &hover_materials.yellow,
                crate::rubik::components::Face::Left => &hover_materials.orange,
                crate::rubik::components::Face::Right => &hover_materials.red,
                crate::rubik::components::Face::Front => &hover_materials.green,
                crate::rubik::components::Face::Back => &hover_materials.blue,
            };
            *material = MeshMaterial3d(hover_mat.clone());
            commands.entity(entity).insert(HandHovered);
        }
    }
}
