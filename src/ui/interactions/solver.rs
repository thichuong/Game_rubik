use super::InteractionQuery;
use crate::events::ResetCameraEvent;
use crate::rubik::components::{
    Cubie, CubieFace, Direction, GridCoord, Pivot, RotationAxis, RotationMove, RubikCube,
};
use crate::rubik::resources::{
    CurrentlyRotating, FaceMapping, MoveHistory, RotationQueue, RubikSize,
};
use crate::ui::components::{
    CloseButton, NextStepButton, RunAllButton, ShuffleButton, SolutionPanel, SolveButton,
    SolveButtonText, StepText,
};
use bevy::prelude::*;
use bevy::tasks::Task;
use rand::RngExt;
use rubik_solver::{SolverResource, StepByStepSolution, helpers};
use std::fmt::Write;

// A resource wrapping the asynchronous task currently computing the Rubik's cube solution
#[derive(Resource)]
pub struct SolverTask(pub Task<Option<Vec<String>>>);

pub fn handle_shuffle_button(
    mut interaction_query: InteractionQuery<ShuffleButton>,
    mut rotation_queue: ResMut<RotationQueue>,
    rubik_size: Res<RubikSize>,
    mut solution: ResMut<StepByStepSolution>,
) {
    // Disable shuffling if solver is actively searching or still has steps to execute
    if solution.is_searching || (solution.active && solution.current_step < solution.moves.len()) {
        return;
    }

    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if solution.active {
                    solution.active = false;
                }
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.35, 0.55, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 0.6, 0.9, 1.0)));

                let mut rng = rand::rng();
                let size = rubik_size.size;
                for _ in 0..20 {
                    let axis = match rng.random_range(0..3) {
                        0 => RotationAxis::X,
                        1 => RotationAxis::Y,
                        _ => RotationAxis::Z,
                    };

                    // Generate a random slice index between 0 and size-1, avoiding the center slice for odd sizes
                    let mut index = rng.random_range(0..size);
                    if size % 2 != 0 {
                        while index == size / 2 {
                            index = rng.random_range(0..size);
                        }
                    }

                    let direction = if rng.random_bool(0.5) {
                        Direction::Clockwise
                    } else {
                        Direction::CounterClockwise
                    };

                    rotation_queue.0.push_back(RotationMove {
                        axis,
                        index,
                        direction,
                        add_to_history: true,
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.18, 0.22, 0.38, 0.95)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.35, 0.45, 0.7, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.15, 0.25, 0.85)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.3, 0.5, 0.6)));
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
pub fn handle_solve_button(
    mut commands: Commands,
    mut interaction_query: InteractionQuery<SolveButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut reset_camera: MessageWriter<ResetCameraEvent>,
    faces: Query<(&CubieFace, &GlobalTransform)>,
    cube_query: Single<&GlobalTransform, With<RubikCube>>,
    solver_res: Res<SolverResource>,
    rubik_size: Res<RubikSize>,
    face_mapping: Res<FaceMapping>,
    mut history: ResMut<MoveHistory>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    // Anti-spam: ignore clicks if the solver is already computing a solution in the background
    if solution.is_searching {
        return;
    }

    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.35, 0.5, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                reset_camera.write(ResetCameraEvent);

                // Stop any pending rotation animations instantly to prevent state mismatch
                rotation_queue.0.clear();

                solution.active = true;
                solution.is_searching = true;
                solution.moves.clear();
                solution.current_step = 0;
                solution.failed = false;

                let size = rubik_size.size;

                // Scrape the Rubik's cube state safely on the main thread since queries cannot cross threads
                if let Some(state_str) =
                    helpers::get_cube_state_for_size(size, &faces, &cube_query, *face_mapping)
                {
                    let thread_pool = bevy::tasks::AsyncComputeTaskPool::get();
                    let table_arc = solver_res.table.clone();

                    // Spawn the solver inside Bevy's async task pool to prevent the main thread from freezing
                    let task = thread_pool.spawn(async move {
                        rubik_solver::solver::solve_cube_for_size(size, &state_str, &table_arc)
                    });

                    commands.insert_resource(SolverTask(task));
                } else {
                    solution.failed = true;
                    solution.is_searching = false;
                }

                // Clear done and undone history upon solving to prevent conflicts
                history.done.clear();
                history.undone.clear();
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.32, 0.22, 0.95)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.7, 0.45, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.22, 0.15, 0.85)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.6)));
            }
        }
    }
}

// System to poll the background solver task and commit the moves to StepByStepSolution
pub fn poll_solver_task(
    mut commands: Commands,
    solver_task: Option<ResMut<SolverTask>>,
    mut solution: ResMut<StepByStepSolution>,
) {
    let Some(mut task) = solver_task else {
        return;
    };

    // Use non-blocking poll_once to check if the async task has finished executing
    if let Some(result) =
        bevy::tasks::block_on(bevy::tasks::futures_lite::future::poll_once(&mut task.0))
    {
        solution.is_searching = false;

        if let Some(moves) = result {
            solution.moves = moves;
            if solution.moves.is_empty() {
                // Already solved -> hide panel
                solution.active = false;
            }
        } else {
            solution.failed = true;
        }

        // Clean up the task resource once it has run to completion
        commands.remove_resource::<SolverTask>();
    }
}

pub fn handle_next_step_button(
    mut interaction_query: InteractionQuery<NextStepButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut rotation_queue: ResMut<RotationQueue>,
    rubik_size: Res<RubikSize>,
    face_mapping: Res<FaceMapping>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.5, 0.35, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let move_str = &solution.moves[solution.current_step];
                    let moves = helpers::logical_string_to_physical_moves_any(
                        move_str,
                        rubik_size.size,
                        *face_mapping,
                    );
                    for m in moves {
                        rotation_queue.0.push_back(m);
                    }
                    solution.current_step += 1;
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.32, 0.22, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.7, 0.45, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.22, 0.15, 0.9)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.6)));
            }
        }
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn handle_run_all_button(
    mut commands: Commands,
    mut interaction_query: InteractionQuery<RunAllButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut rotation_queue: ResMut<RotationQueue>,
    rubik_size: Res<RubikSize>,
    face_mapping: Res<FaceMapping>,
    mut cubies: Query<(&mut Transform, &mut GridCoord), With<Cubie>>,
    cube_root: Single<Entity, With<RubikCube>>,
    current_rotating: Option<Res<CurrentlyRotating>>,
    pivot_query: Query<Entity, With<Pivot>>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.4, 0.55, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.7, 0.9, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let size = rubik_size.size;
                    let root_entity = *cube_root;

                    // 1. Instantly complete current rotation animation if there is one in progress
                    if let Some(ref current) = current_rotating {
                        for &cubie_entity in &current.cubies {
                            if let Ok((mut transform, mut coord)) = cubies.get_mut(cubie_entity) {
                                coord.rotate(current.rotation_axis, current.angle, size);

                                let offset = (size as f32 - 1.0) / 2.0;
                                let scale = 3.0 / size as f32;
                                let current_gap = crate::rubik::systems::creation::GAP * scale;

                                transform.translation =
                                    (coord.0.as_vec3() - Vec3::splat(offset)) * current_gap;
                                transform.scale = Vec3::splat(scale);

                                let rot_step =
                                    Quat::from_axis_angle(current.rotation_axis, current.angle);
                                transform.rotation = (rot_step * transform.rotation).normalize();

                                commands.entity(cubie_entity).insert(ChildOf(root_entity));
                            }
                        }

                        for p_entity in &pivot_query {
                            commands.entity(p_entity).despawn();
                        }
                        commands.remove_resource::<CurrentlyRotating>();
                    }

                    // Clear any queued animated moves to prevent delayed execution
                    rotation_queue.0.clear();

                    // 2. Process all remaining moves instantly on all cubies
                    let offset = (size as f32 - 1.0) / 2.0;
                    let scale = 3.0 / size as f32;
                    let current_gap = crate::rubik::systems::creation::GAP * scale;

                    for i in solution.current_step..solution.moves.len() {
                        let move_str = &solution.moves[i];
                        let moves = helpers::logical_string_to_physical_moves_any(
                            move_str,
                            size,
                            *face_mapping,
                        );

                        for m in moves {
                            let (axis_vec, angle) = m.get_rotation_info();

                            for (mut transform, mut coord) in &mut cubies {
                                if m.is_cubie_at_slice(coord.0) {
                                    coord.rotate(axis_vec, angle, size);

                                    let rot_step = Quat::from_axis_angle(axis_vec, angle);
                                    transform.rotation =
                                        (rot_step * transform.rotation).normalize();
                                    transform.translation =
                                        (coord.0.as_vec3() - Vec3::splat(offset)) * current_gap;
                                }
                            }
                        }
                    }

                    solution.current_step = solution.moves.len();
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.22, 0.32, 0.9)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.5, 0.65, 0.7)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.18, 0.25, 0.9)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.4, 0.5, 0.6)));
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn update_solution_panel(
    solution: Res<StepByStepSolution>,
    mut panel: Single<&mut Node, With<SolutionPanel>>,
    mut text: Single<&mut Text, With<StepText>>,
    time: Res<Time>,
    mut anim_timer: Local<f32>,
) {
    if solution.is_changed() || solution.is_searching {
        panel.display = if solution.active
            && (!solution.moves.is_empty() || solution.failed || solution.is_searching)
        {
            Display::Flex
        } else {
            Display::None
        };

        if solution.active {
            if solution.is_searching {
                // Update the visual character spinner animation smoothly based on delta time
                *anim_timer += time.delta_secs();
                let spinner = match ((*anim_timer * 6.0) as usize) % 4 {
                    0 => "|",
                    1 => "/",
                    2 => "-",
                    _ => "\\",
                };
                text.0 = format!("Analyzing and solving cube state... [{spinner}]");
            } else if solution.failed {
                text.0 = "Failed to solve! (Invalid state)".to_string();
            } else if !solution.moves.is_empty() {
                if solution.current_step >= solution.moves.len() {
                    text.0 = "Solved!".to_string();
                } else {
                    let total_moves = solution.moves.len();
                    let current = solution.current_step;

                    // Implement a sliding window of size 7 around current_step to prevent UI overflow and lag.
                    // This shows up to 3 moves before, the current highlighted move, and up to 3 moves after.
                    let window_half = 3;
                    let start = current.saturating_sub(window_half);
                    let end = (current + window_half + 1).min(total_moves);

                    let mut window_text = String::new();

                    if start > 0 {
                        let _ = write!(window_text, "... ");
                    }

                    for i in start..end {
                        let m = &solution.moves[i];
                        if i == current {
                            let _ = write!(window_text, " >>{m}<< ");
                        } else {
                            let _ = write!(window_text, " {m} ");
                        }
                    }

                    if end < total_moves {
                        let _ = write!(window_text, " ...");
                    }

                    text.0 = format!("Step {}/{}\n\n{}", current + 1, total_moves, window_text);
                }
            }
        }
    }
}

pub fn handle_close_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
    mut solution: ResMut<StepByStepSolution>,
) {
    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) {
            solution.active = false;
        }
    }
}

/// System to dynamically enable/disable the Solve Button state (gray out if size is not 3x3x3)
pub fn update_solve_button_state(
    rubik_size: Res<RubikSize>,
    solution: Res<StepByStepSolution>,
    solve_btn_query: Single<(&mut BackgroundColor, &mut BorderColor), With<SolveButton>>,
    mut text_query: Single<&mut Text, With<SolveButtonText>>,
) {
    if rubik_size.is_changed() || solution.is_changed() {
        let (mut bg, mut border) = solve_btn_query.into_inner();
        if solution.is_searching {
            // Darken and grey out the solve button when currently computing in background
            *bg = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.18, 0.85)));
            *border = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.3, 0.35, 0.5)));
            text_query.0 = "SOLVING...".to_string();
        } else {
            // Restore beautiful green theme for active solver on all sizes
            *bg = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.22, 0.15, 0.85)));
            *border = BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.6)));
            text_query.0 = "SOLVE".to_string();
        }
    }
}
