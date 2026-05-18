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

#[derive(Resource, Default)]
pub struct HandDragState {
    pub start_face: Option<(Entity, Vec3, Vec3)>, // (Entity, normal, hit_point)
    pub prev_gesture_type: u8,
}

#[derive(Component)]
struct VirtualHandJoint {
    handedness: u8, // 0 = Left, 1 = Right
    joint_index: usize,
}

#[derive(Resource)]
pub struct HandVisuals {
    pub joint_mesh: Handle<Mesh>,
    pub mat_left: Handle<StandardMaterial>, // Neon Purple/Pink for Left Hand
    pub mat_right_orbit: Handle<StandardMaterial>, // Neon Purple/Blue for Right Hand whole hand
    pub mat_right_interact: Handle<StandardMaterial>, // Neon Cyan/Green for Right Hand index pointing
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HandTrackingEnabled(false))
            .insert_resource(HandTrackingStateResource::default())
            .insert_resource(HandDragState::default())
            .add_message::<HandRotationEvent>()
            .add_message::<CameraFrameEvent>()
            .add_systems(Startup, (setup_camera_listener, setup_hand_visuals))
            .add_systems(
                Update,
                (receive_hand_tracking, update_virtual_hands, draw_hand_bones),
            );
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

fn setup_hand_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Elegant small spheres for joint rendering
    let joint_mesh = meshes.add(Sphere::new(0.06));

    // Glow neon purple for left hand
    let mat_left = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::new(0.8, 0.1, 0.9, 0.8)),
        emissive: LinearRgba::new(8.0, 1.0, 9.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Glow neon purple/blue for right hand when orbiting
    let mat_right_orbit = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::new(0.1, 0.6, 0.9, 0.8)),
        emissive: LinearRgba::new(1.0, 6.0, 9.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Glow neon cyan/green for right hand when interacting/pointing
    let mat_right_interact = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::new(0.1, 0.9, 0.4, 0.8)),
        emissive: LinearRgba::new(1.0, 9.0, 4.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(HandVisuals {
        joint_mesh,
        mat_left,
        mat_right_orbit,
        mat_right_interact,
    });
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
            if hands.len() == 2 {
                // 2-Hand Mode
                for hand in hands {
                    if hand.handedness == 0 {
                        // Left Hand: Orbit cube
                        if let Some((dx, dy)) = hand.delta {
                            rot_events.write(HandRotationEvent {
                                delta_x: dx,
                                delta_y: dy,
                            });
                        }
                    } else if hand.handedness == 1 {
                        // Right Hand: Rotate face
                        if hand.gesture_type == 2 {
                            process_face_drag(
                                hand.gesture_type,
                                hand.cursor,
                                data.width,
                                data.height,
                                &mut drag_state,
                                window,
                                camera,
                                camera_transform,
                                &cubie_faces,
                            );
                        } else {
                            if drag_state.prev_gesture_type == 2 {
                                process_face_drag_release(
                                    hand.cursor,
                                    data.width,
                                    data.height,
                                    &mut drag_state,
                                    &mut rotation_queue,
                                    window,
                                    camera,
                                    camera_transform,
                                    &cubie_faces,
                                    cube_transform,
                                    *rubik_size,
                                );
                            }
                            drag_state.prev_gesture_type = hand.gesture_type;
                        }
                    }
                }
            } else if hands.len() == 1 {
                // 1-Hand Mode
                let hand = &hands[0];
                if hand.gesture_type == 1 {
                    // Orbit cube
                    if let Some((dx, dy)) = hand.delta {
                        rot_events.write(HandRotationEvent {
                            delta_x: dx,
                            delta_y: dy,
                        });
                    }
                    if drag_state.prev_gesture_type == 2 {
                        process_face_drag_release(
                            hand.cursor,
                            data.width,
                            data.height,
                            &mut drag_state,
                            &mut rotation_queue,
                            window,
                            camera,
                            camera_transform,
                            &cubie_faces,
                            cube_transform,
                            *rubik_size,
                        );
                    }
                    drag_state.prev_gesture_type = 1;
                } else if hand.gesture_type == 2 {
                    // Rotate face
                    process_face_drag(
                        hand.gesture_type,
                        hand.cursor,
                        data.width,
                        data.height,
                        &mut drag_state,
                        window,
                        camera,
                        camera_transform,
                        &cubie_faces,
                    );
                }
            } else {
                // Hand lost: reset drag
                if drag_state.prev_gesture_type == 2 {
                    drag_state.start_face = None;
                    drag_state.prev_gesture_type = 0;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::cast_precision_loss)]
fn process_face_drag(
    gesture_type: u8,
    cursor: (f32, f32),
    tracker_w: u32,
    tracker_h: u32,
    drag_state: &mut HandDragState,
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

    if gesture_type == 2 {
        if drag_state.prev_gesture_type != 2 {
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
        drag_state.prev_gesture_type = 2;
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn process_face_drag_release(
    cursor: (f32, f32),
    tracker_w: u32,
    tracker_h: u32,
    drag_state: &mut HandDragState,
    rotation_queue: &mut RotationQueue,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cubie_faces: &Query<(Entity, &CubieFace, &GlobalTransform)>,
    cube_transform: &GlobalTransform,
    rubik_size: RubikSize,
) {
    if let Some((start_entity, start_normal, start_hit_point)) = drag_state.start_face {
        let window_width = window.width();
        let window_height = window.height();
        let mapped_x = (cursor.0 / tracker_w as f32) * window_width;
        let mapped_y = (cursor.1 / tracker_h as f32) * window_height;
        let mapped_cursor = Vec2::new(mapped_x, mapped_y);

        let Ok(ray) = camera.viewport_to_world(camera_transform, mapped_cursor) else {
            drag_state.start_face = None;
            return;
        };

        let denom = ray.direction.dot(start_normal);
        if denom.abs() > 1e-6 {
            let t = (start_hit_point.dot(start_normal) - ray.origin.dot(start_normal)) / denom;
            let end_hit_point = ray.origin + *ray.direction * t;
            let drag_vec = end_hit_point - start_hit_point;

            if drag_vec.length() > 0.3 {
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
            }
        }
    }
    drag_state.start_face = None;
}

#[allow(clippy::type_complexity, clippy::suboptimal_flops)]
fn update_virtual_hands(
    mut commands: Commands,
    visuals: Res<HandVisuals>,
    camera_query: Query<Entity, With<Camera3d>>,
    mut joints_query: Query<(
        Entity,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
        &VirtualHandJoint,
    )>,
    hands_state: Res<HandTrackingStateResource>,
    enabled: Res<HandTrackingEnabled>,
) {
    let Some(camera_ent) = camera_query.iter().next() else {
        return;
    };

    let left_joints_exist = joints_query.iter().any(|(_, _, _, _, j)| j.handedness == 0);
    let right_joints_exist = joints_query.iter().any(|(_, _, _, _, j)| j.handedness == 1);

    if !left_joints_exist {
        commands.entity(camera_ent).with_children(|parent| {
            for i in 0..21 {
                parent.spawn((
                    VirtualHandJoint {
                        handedness: 0,
                        joint_index: i,
                    },
                    Mesh3d(visuals.joint_mesh.clone()),
                    MeshMaterial3d(visuals.mat_left.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ));
            }
        });
    }

    if !right_joints_exist {
        commands.entity(camera_ent).with_children(|parent| {
            for i in 0..21 {
                parent.spawn((
                    VirtualHandJoint {
                        handedness: 1,
                        joint_index: i,
                    },
                    Mesh3d(visuals.joint_mesh.clone()),
                    MeshMaterial3d(visuals.mat_right_orbit.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ));
            }
        });
    }

    if !enabled.0 {
        for (_, _, _, mut vis, _) in &mut joints_query {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let mut left_active = false;
    let mut right_active = false;

    let width_scale = 3.5;
    let height_scale = 2.5;
    let depth_scale = 3.0;
    let distance_from_camera = 4.0;

    for hand in &hands_state.hands {
        if hand.handedness == 0 {
            left_active = true;
        } else {
            right_active = true;
        }

        for (_, mut transform, mut material, mut vis, joint) in &mut joints_query {
            if joint.handedness == hand.handedness {
                if let Some(lm) = hand.landmarks.get(joint.joint_index) {
                    *vis = Visibility::Visible;

                    let local_x = (lm.x - 0.5) * width_scale;
                    let local_y = (0.5 - lm.y) * height_scale;
                    let local_z = -distance_from_camera + (lm.z * depth_scale);

                    transform.translation = Vec3::new(local_x, local_y, local_z);

                    if joint.handedness == 0 {
                        *material = MeshMaterial3d(visuals.mat_left.clone());
                    } else if hand.gesture_type == 2 {
                        *material = MeshMaterial3d(visuals.mat_right_interact.clone());
                    } else {
                        *material = MeshMaterial3d(visuals.mat_right_orbit.clone());
                    }
                }
            }
        }
    }

    if !left_active {
        for (_, _, _, mut vis, joint) in &mut joints_query {
            if joint.handedness == 0 {
                *vis = Visibility::Hidden;
            }
        }
    }

    if !right_active {
        for (_, _, _, mut vis, joint) in &mut joints_query {
            if joint.handedness == 1 {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn draw_hand_bones(
    joints_query: Query<(&GlobalTransform, &Visibility, &VirtualHandJoint)>,
    mut gizmos: Gizmos,
) {
    const HAND_CONNECTIONS: [(usize, usize); 21] = [
        // Palm / wrist
        (0, 1),
        (0, 5),
        (0, 17),
        // Thumb
        (1, 2),
        (2, 3),
        (3, 4),
        // Index finger
        (5, 6),
        (6, 7),
        (7, 8),
        // Middle finger
        (9, 10),
        (10, 11),
        (11, 12),
        // Ring finger
        (13, 14),
        (14, 15),
        (15, 16),
        // Pinky
        (17, 18),
        (18, 19),
        (19, 20),
        // Knuckles
        (5, 9),
        (9, 13),
        (13, 17),
    ];

    let mut left_positions = vec![None; 21];
    let mut right_positions = vec![None; 21];
    let mut left_visible = false;
    let mut right_visible = false;

    for (gt, vis, joint) in &joints_query {
        if matches!(vis, Visibility::Visible) {
            let pos = gt.translation();
            if joint.handedness == 0 {
                left_positions[joint.joint_index] = Some(pos);
                left_visible = true;
            } else {
                right_positions[joint.joint_index] = Some(pos);
                right_visible = true;
            }
        }
    }

    if left_visible {
        let color = Color::Srgba(Srgba::new(0.8, 0.1, 0.9, 0.6));
        for &(a, b) in &HAND_CONNECTIONS {
            if let (Some(pa), Some(pb)) = (left_positions[a], left_positions[b]) {
                gizmos.line(pa, pb, color);
            }
        }
    }

    if right_visible {
        let color = Color::Srgba(Srgba::new(0.1, 0.6, 0.9, 0.6));
        for &(a, b) in &HAND_CONNECTIONS {
            if let (Some(pa), Some(pb)) = (right_positions[a], right_positions[b]) {
                gizmos.line(pa, pb, color);
            }
        }
    }
}
