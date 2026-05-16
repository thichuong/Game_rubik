use crate::input::resources::DragState;
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove};
use crate::rubik::resources::{MoveHistory, RotationQueue};
use bevy::prelude::*;
// Removed PickingInteraction import

pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    mut rotation_queue: ResMut<RotationQueue>,
    picks: Query<(Entity, &CubieFace, &GlobalTransform, &Interaction)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (entity, _cubie_face, transform, interaction) in picks.iter() {
            if *interaction == Interaction::Pressed {
                let normal: Vec3 = transform.back().into();
                let pos = transform.translation();
                drag_state.start_face = Some((entity, normal, pos));
                break;
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if let Some((_start_entity, start_normal, start_pos)) = drag_state.start_face {
            for (_entity, _cubie_face, transform, interaction) in picks.iter() {
                if *interaction == Interaction::Hovered {
                    let end_pos: Vec3 = transform.translation();
                    let drag_vec = end_pos - start_pos;

                    if drag_vec.length() > 0.5 {
                        let drag_dir = drag_vec.normalize();
                        let rotation_axis_vec = start_normal.cross(drag_dir);

                        // Find closest primary axis
                        let mut best_axis = RotationAxis::X;
                        let mut max_dot = 0.0;

                        for axis in [RotationAxis::X, RotationAxis::Y, RotationAxis::Z] {
                            let dot = rotation_axis_vec.dot(axis.vector()).abs();
                            if dot > max_dot {
                                max_dot = dot;
                                best_axis = axis;
                            }
                        }

                        let axis_vec = best_axis.vector();
                        let sign = rotation_axis_vec.dot(axis_vec).signum();

                        let direction = if sign > 0.0 {
                            Direction::CounterClockwise
                        } else {
                            Direction::Clockwise
                        };

                        #[allow(clippy::cast_possible_truncation)]
                        let index = match best_axis {
                            RotationAxis::X => start_pos.x.round() as i32,
                            RotationAxis::Y => start_pos.y.round() as i32,
                            RotationAxis::Z => start_pos.z.round() as i32,
                        };

                        rotation_queue.0.push_back(RotationMove {
                            axis: best_axis,
                            index,
                            direction,
                            add_to_history: true,
                        });
                    }
                    break;
                }
            }
        }
        drag_state.start_face = None;
    }
}

pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rotation_queue: ResMut<RotationQueue>,
    mut history: ResMut<MoveHistory>,
) {
    // Undo logic
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyZ) {
        if let Some(last_move) = history.done.pop() {
            let inverse_move = last_move.inverse();
            rotation_queue.0.push_back(inverse_move);
            history.undone.push(last_move);
        }
    }

    // Redo logic
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyY) {
        if let Some(last_undone) = history.undone.pop() {
            let mut redo_move = last_undone;
            redo_move.add_to_history = true;
            rotation_queue.0.push_back(redo_move);
        }
    }
}
