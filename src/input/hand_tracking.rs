use crate::events::{CameraFrameEvent, HandRotationEvent};
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{RotationQueue, RubikSize};
use crate::rubik::systems::creation::GAP;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
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

#[derive(Resource, Default)]
pub struct HandTrackingStateResource {
    pub hands: Vec<hand_tracker::TrackerHand>,
}

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
            .insert_resource(HandTrackingStateResource::default())
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
            // ~60 FPS processing rate
            thread::sleep(std::time::Duration::from_millis(16));
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
    mut hands_state: ResMut<HandTrackingStateResource>,
    mut drag_state: ResMut<HandDragState>,
    mut rotation_queue: ResMut<RotationQueue>,
    mut rot_events: MessageWriter<HandRotationEvent>,
    mut frame_events: MessageWriter<CameraFrameEvent>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    cubie_faces: Query<(Entity, &CubieFace, &GlobalTransform)>,
    cube_query: Single<&GlobalTransform, With<RubikCube>>,
    rubik_size: Res<RubikSize>,
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

    for data in rx.try_iter() {
        // Send camera frame to UI
        frame_events.write(CameraFrameEvent {
            frame_rgba: data.frame_rgba,
            width: data.width,
            height: data.height,
        });

        // Store latest hands state for 3D visual rendering
        hands_state.hands = data.hands;

        if enabled.0 {
            let hands = &hands_state.hands;
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
                    if drag_sub_state.prev_gesture_type == 2 {
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
                    // Index Pointing: Rotate face in real-time
                    process_face_active_drag(
                        hand.gesture_type,
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
                } else {
                    // Clean up face drag for other/unknown gestures
                    if drag_sub_state.prev_gesture_type == 2 {
                        drag_sub_state.start_face = None;
                    }
                    drag_sub_state.prev_gesture_type = hand.gesture_type;
                }
            }

            // Clean up drag state for hands that disappeared in this frame
            if !left_active && drag_state.left.prev_gesture_type == 2 {
                drag_state.left.start_face = None;
                drag_state.left.prev_gesture_type = 0;
            }
            if !right_active && drag_state.right.prev_gesture_type == 2 {
                drag_state.right.start_face = None;
                drag_state.right.prev_gesture_type = 0;
            }
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
fn process_face_active_drag(
    gesture_type: u8,
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
    let window_width = window.width();
    let window_height = window.height();
    let mapped_x = (cursor.0 / tracker_w as f32) * window_width;
    let mapped_y = (cursor.1 / tracker_h as f32) * window_height;
    let mapped_cursor = Vec2::new(mapped_x, mapped_y);

    let Ok(ray) = camera.viewport_to_world(camera_transform, mapped_cursor) else {
        return;
    };

    if gesture_type == 2 {
        if drag_state.prev_gesture_type != 2 || drag_state.start_face.is_none() {
            // Step 1: Detect starting face when we just entered Gesture 2 (Index Pointing)
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
        } else if let Some((start_entity, start_normal, start_hit_point)) = drag_state.start_face {
            // Step 2: Track active dragging relative to start point
            let denom = ray.direction.dot(start_normal);
            if denom.abs() > 1e-6 {
                let t = (start_hit_point.dot(start_normal) - ray.origin.dot(start_normal)) / denom;
                let end_hit_point = ray.origin + *ray.direction * t;
                let drag_vec = end_hit_point - start_hit_point;

                // Emulate drag-to-rotate in real-time if movement threshold is exceeded
                if drag_vec.length() > 0.35 {
                    let drag_dir = drag_vec.normalize();
                    let rotation_axis_vec = start_normal.cross(drag_dir);

                    let mut best_axis = RotationAxis::X;
                    let mut max_dot = 0.0;
                    let mut best_local_axis_in_world = Vec3::X;

                    for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
                        let local_axis_in_world =
                            cube_transform.affine().transform_vector3(axis.vector());
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
                                RotationAxis::X => {
                                    ((local_pos.x / current_gap) + offset).round() as i32
                                }
                                RotationAxis::Y => {
                                    ((local_pos.y / current_gap) + offset).round() as i32
                                }
                                RotationAxis::Z => {
                                    ((local_pos.z / current_gap) + offset).round() as i32
                                }
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

                    // Fluid rotation: update start position to current hit point so user can drag continuously
                    drag_state.start_face = Some((start_entity, start_normal, end_hit_point));
                }
            }
        }
        drag_state.prev_gesture_type = 2;
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

#[allow(
    clippy::type_complexity,
    clippy::suboptimal_flops,
    clippy::too_many_lines
)]
fn update_hand_hover(
    mut commands: Commands,
    enabled: Res<HandTrackingEnabled>,
    hands_state: Res<HandTrackingStateResource>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    rubik_size: Res<RubikSize>,
    cubie_query: Query<
        (
            Entity,
            &crate::rubik::components::GridCoord,
            &GlobalTransform,
            &Children,
        ),
        With<crate::rubik::components::Cubie>,
    >,
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

    // 2. If hand tracking is disabled or no hands are visible, stop here
    if !enabled.0 || hands_state.hands.is_empty() {
        return;
    }

    let Some(camera_transform) = camera_query.iter().next() else {
        return;
    };

    let size = rubik_size.size;
    let width_scale = 3.5;
    let height_scale = 2.5;
    let depth_scale = 3.0;
    let distance_from_camera = 4.0;

    // Helper closure to identify corner cubies in NxNxN logical grid
    let is_corner = |coord: IVec3| -> bool {
        let max_val = size - 1;
        (coord.x == 0 || coord.x == max_val)
            && (coord.y == 0 || coord.y == max_val)
            && (coord.z == 0 || coord.z == max_val)
    };

    // Helper closure to identify edge cubies in NxNxN logical grid (exactly 2 coords are at borders)
    let is_edge = |coord: IVec3| -> bool {
        let max_val = size - 1;
        let count = i32::from(coord.x == 0 || coord.x == max_val)
            + i32::from(coord.y == 0 || coord.y == max_val)
            + i32::from(coord.z == 0 || coord.z == max_val);
        count == 2
    };

    for hand in &hands_state.hands {
        // Thumb (4) and Index (8) are required to initiate the hand hover simulator
        let Some(lm4) = hand.landmarks.get(4) else {
            continue;
        };
        let Some(lm8) = hand.landmarks.get(8) else {
            continue;
        };

        // Project finger 2D coordinates into 3D world space camera plane
        let thumb_local = Vec3::new(
            (lm4.x - 0.5) * width_scale,
            (0.5 - lm4.y) * height_scale,
            -distance_from_camera + (lm4.z * depth_scale),
        );
        let thumb_world = camera_transform.transform_point(thumb_local);

        let index_local = Vec3::new(
            (lm8.x - 0.5) * width_scale,
            (0.5 - lm8.y) * height_scale,
            -distance_from_camera + (lm8.z * depth_scale),
        );
        let index_world = camera_transform.transform_point(index_local);

        let mut best_thumb_corner = None;
        let mut min_thumb_dist = f32::MAX;

        let mut best_index_corner = None;
        let mut min_index_dist = f32::MAX;

        // Process other finger tips (Middle: 12, Ring: 16, Pinky: 20)
        let lm12 = hand.landmarks.get(12);
        let lm16 = hand.landmarks.get(16);
        let lm20 = hand.landmarks.get(20);

        let mut other_fingers_world = Vec::with_capacity(3);
        for lm in [lm12, lm16, lm20].into_iter().flatten() {
            let local_pos = Vec3::new(
                (lm.x - 0.5) * width_scale,
                (0.5 - lm.y) * height_scale,
                -distance_from_camera + (lm.z * depth_scale),
            );
            other_fingers_world.push(camera_transform.transform_point(local_pos));
        }

        let mut other_fingers_best_cubie = vec![None; other_fingers_world.len()];
        let mut other_fingers_min_dist = vec![f32::MAX; other_fingers_world.len()];

        // Find the closest cubies
        for (cubie_ent, coord, transform, children) in &cubie_query {
            let cubie_pos = transform.translation();
            let coord_vec = coord.0;

            if is_corner(coord_vec) {
                let d_thumb = cubie_pos.distance(thumb_world);
                if d_thumb < min_thumb_dist {
                    min_thumb_dist = d_thumb;
                    best_thumb_corner = Some((cubie_ent, children));
                }

                let d_index = cubie_pos.distance(index_world);
                if d_index < min_index_dist {
                    min_index_dist = d_index;
                    best_index_corner = Some((cubie_ent, children));
                }
            }

            if is_corner(coord_vec) || is_edge(coord_vec) {
                for (i, &f_world) in other_fingers_world.iter().enumerate() {
                    let d = cubie_pos.distance(f_world);
                    if d < other_fingers_min_dist[i] {
                        other_fingers_min_dist[i] = d;
                        other_fingers_best_cubie[i] = Some((cubie_ent, children));
                    }
                }
            }
        }

        let mut cubies_to_hover = Vec::new();
        if let Some((_, children)) = best_thumb_corner {
            cubies_to_hover.push(children);
        }
        if let Some((_, children)) = best_index_corner {
            cubies_to_hover.push(children);
        }

        // Other fingers only hover if close enough to simulate a natural touch
        for (i, &min_d) in other_fingers_min_dist.iter().enumerate() {
            if min_d < 1.3 {
                if let Some((_, children)) = &other_fingers_best_cubie[i] {
                    cubies_to_hover.push(*children);
                }
            }
        }

        // Apply glowing hover materials to all faces of the selected cubies
        for children in cubies_to_hover {
            for &child in children {
                if let Ok((_ent, face, mut material, _hovered)) = face_material_query.get_mut(child)
                {
                    let hover_mat = match face.0 {
                        crate::rubik::components::Face::Up => &hover_materials.white,
                        crate::rubik::components::Face::Down => &hover_materials.yellow,
                        crate::rubik::components::Face::Left => &hover_materials.orange,
                        crate::rubik::components::Face::Right => &hover_materials.red,
                        crate::rubik::components::Face::Front => &hover_materials.green,
                        crate::rubik::components::Face::Back => &hover_materials.blue,
                    };
                    *material = MeshMaterial3d(hover_mat.clone());
                    commands.entity(child).insert(HandHovered);
                }
            }
        }
    }
}
