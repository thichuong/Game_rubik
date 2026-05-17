use crate::environment::resources::EnvironmentSettings;
use crate::events::ResetCameraEvent;
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{MoveHistory, RotationQueue, RubikSize, RubikSkin};
use crate::solver::helpers;
use crate::solver::resources::{SolverResource, StepByStepSolution};
use crate::ui::components::{
    CloseButton, EnvControl, EnvList, EnvToggleButton, NextStepButton, ShuffleButton,
    SizeDecrementButton, SizeIncrementButton, SizeSliderFill, SizeSliderHandle, SizeSliderTrack,
    SizeText, SkinButton, SkinList, SkinToggleButton, SolutionPanel, SolveButton, SolveButtonText,
    StepText,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::RngExt;
use std::fmt::Write;

pub type InteractionQuery<'w, 's, T> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<Interaction>, With<T>),
>;

pub fn handle_shuffle_button(
    mut interaction_query: InteractionQuery<ShuffleButton>,
    mut rotation_queue: ResMut<RotationQueue>,
    rubik_size: Res<RubikSize>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.3, 0.55, 1.0)));
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

pub fn handle_solve_button(
    mut interaction_query: InteractionQuery<SolveButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut reset_camera: MessageWriter<ResetCameraEvent>,
    faces: Query<(&CubieFace, &GlobalTransform)>,
    cube_query: Single<&GlobalTransform, With<RubikCube>>,
    solver_res: Res<SolverResource>,
    rubik_size: Res<RubikSize>,
    mut history: ResMut<MoveHistory>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.35, 0.5, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                reset_camera.write(ResetCameraEvent);

                // Stop any pending rotation animations instantly to prevent state mismatch
                rotation_queue.0.clear();

                solution.active = true;
                solution.moves.clear();
                solution.current_step = 0;

                let size = rubik_size.size;

                if size == 3 {
                    let state_str = helpers::get_cube_state(&faces, &cube_query);

                    let mut solved_with_kewb = false;
                    if let Ok(face_cube) = kewb::FaceCube::try_from(state_str.as_str()) {
                        if let Ok(cubie_cube) = kewb::CubieCube::try_from(&face_cube) {
                            let mut solver = kewb::Solver::new(&solver_res.table, 23, None);
                            if let Some(sol) = solver.solve(cubie_cube) {
                                solution.moves = sol
                                    .to_string()
                                    .split_whitespace()
                                    .map(String::from)
                                    .collect();
                                solved_with_kewb = true;
                            }
                        }
                    }

                    if !solved_with_kewb && !history.done.is_empty() {
                        solution.moves = history
                            .done
                            .iter()
                            .rev()
                            .map(|m| helpers::move_to_string(m.inverse(), size))
                            .collect();
                    }
                } else if !history.done.is_empty() {
                    solution.moves = history
                        .done
                        .iter()
                        .rev()
                        .map(|m| helpers::move_to_string(m.inverse(), size))
                        .collect();
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

pub fn handle_next_step_button(
    mut interaction_query: InteractionQuery<NextStepButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut rotation_queue: ResMut<RotationQueue>,
    rubik_size: Res<RubikSize>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.5, 0.35, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let move_str = &solution.moves[solution.current_step];
                    let moves = helpers::solution_to_moves(move_str, rubik_size.size);
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

pub fn update_solution_panel(
    solution: Res<StepByStepSolution>,
    mut panel: Single<&mut Node, With<SolutionPanel>>,
    mut text: Single<&mut Text, With<StepText>>,
) {
    if solution.is_changed() {
        panel.display = if solution.active {
            Display::Flex
        } else {
            Display::None
        };

        if solution.active {
            if solution.moves.is_empty() {
                text.0 = "Already solved or no moves recorded!".to_string();
            } else if solution.current_step >= solution.moves.len() {
                text.0 = "Solved!".to_string();
            } else {
                let mut full_text = String::new();
                for (i, m) in solution.moves.iter().enumerate() {
                    if i == solution.current_step {
                        let _ = write!(full_text, " >>{m}<< ");
                    } else {
                        let _ = write!(full_text, " {m} ");
                    }
                }
                text.0 = format!(
                    "Step {}/{}\n\n{}",
                    solution.current_step + 1,
                    solution.moves.len(),
                    full_text
                );
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

pub type SkinButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static SkinButton,
        &'static mut BackgroundColor,
    ),
    (With<Button>, Without<SkinToggleButton>),
>;

pub fn handle_skin_button(
    mut interaction_query: SkinButtonQuery,
    mut rubik_skin: ResMut<RubikSkin>,
) {
    for (interaction, skin_btn, mut bg_color) in &mut interaction_query {
        let is_selected = rubik_skin.current == skin_btn.0;

        match *interaction {
            Interaction::Pressed => {
                rubik_skin.current = skin_btn.0;
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.5, 1.0)));
            }
            Interaction::None => {
                if is_selected {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.5, 0.9, 1.0)));
                } else {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.85)));
                }
            }
        }
    }
}

pub type SkinToggleQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<SkinToggleButton>),
>;

pub fn handle_skin_toggle(
    mut interaction_query: SkinToggleQuery,
    mut skin_list: Single<&mut Node, With<SkinList>>,
    mut state: Local<bool>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *state = !*state;
                skin_list.display = if *state { Display::Flex } else { Display::None };
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.22, 0.22, 0.28, 0.95)));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.18, 0.18, 0.22, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.12, 0.15, 0.6)));
            }
        }
    }
}

pub type EnvToggleQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<EnvToggleButton>),
>;

pub fn handle_env_toggle(
    mut interaction_query: EnvToggleQuery,
    mut env_list: Single<&mut Node, With<EnvList>>,
    mut state: Local<bool>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *state = !*state;
                env_list.display = if *state { Display::Flex } else { Display::None };
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.22, 0.22, 0.28, 0.95)));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.18, 0.18, 0.22, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.12, 0.15, 0.6)));
            }
        }
    }
}

pub type EnvControlQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static EnvControl,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<Button>),
>;

pub fn handle_env_controls(
    mut interaction_query: EnvControlQuery,
    mut settings: ResMut<EnvironmentSettings>,
) {
    for (interaction, control, mut bg_color) in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) {
            match control {
                EnvControl::Intensity(delta) => {
                    settings.light_intensity =
                        (settings.light_intensity + delta).clamp(0.0, 10_000_000.0);
                }
                EnvControl::Temp(color) => {
                    settings.color_temperature = *color;
                }
                EnvControl::Angle(delta) => {
                    settings.light_angle += delta;
                }
                EnvControl::Bg(color) => {
                    settings.background_color = *color;
                }
            }
            *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.5, 1.0)));
        }
    }
}

/// System to handle changes in the Rubik size via the Slider Track
#[allow(clippy::cast_possible_truncation)]
pub fn handle_size_slider_track(
    mut rubik_size: ResMut<RubikSize>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    track_query: Query<
        (&Interaction, &bevy::ui::UiGlobalTransform, &ComputedNode),
        With<SizeSliderTrack>,
    >,
    mut is_dragging: Local<bool>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let Some((track_interaction, transform, computed_node)) = track_query.iter().next() else {
        return;
    };

    if matches!(*track_interaction, Interaction::Pressed) {
        *is_dragging = true;
    }

    if mouse_input.just_released(MouseButton::Left) {
        *is_dragging = false;
    }

    if *is_dragging {
        let window = *window_query;
        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };

        let width = computed_node.size().x;
        let center_x = transform.translation.x;
        let left_x = center_x - width / 2.0;

        let pct = ((cursor_pos.x - left_x) / width).clamp(0.0, 1.0);
        let new_size = 2 + (pct * 10.0).round() as i32;

        if rubik_size.size != new_size {
            rubik_size.size = new_size;
        }
    }
}

/// System to handle fast decrement button (-) for cube size
pub fn handle_size_decrement_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SizeDecrementButton>)>,
    mut rubik_size: ResMut<RubikSize>,
) {
    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) && rubik_size.size > 2 {
            rubik_size.size -= 1;
        }
    }
}

/// System to handle fast increment button (+) for cube size
pub fn handle_size_increment_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SizeIncrementButton>)>,
    mut rubik_size: ResMut<RubikSize>,
) {
    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) && rubik_size.size < 12 {
            rubik_size.size += 1;
        }
    }
}

/// System to update the visual representation of the Slider (fill width & handle position)
#[allow(clippy::cast_precision_loss)]
pub fn update_size_slider_ui(
    rubik_size: Res<RubikSize>,
    mut size_text_query: Single<&mut Text, With<SizeText>>,
    mut fill_query: Single<&mut Node, (With<SizeSliderFill>, Without<SizeSliderHandle>)>,
    mut handle_query: Single<&mut Node, (With<SizeSliderHandle>, Without<SizeSliderFill>)>,
) {
    if rubik_size.is_changed() {
        let size = rubik_size.size;
        size_text_query.0 = format!("{size}x{size}x{size}");

        // Map size 2..=12 to 0%..=100% progress
        let pct = (size - 2) as f32 / 10.0 * 100.0;
        fill_query.width = Val::Percent(pct);
        handle_query.left = Val::Percent(pct);
    }
}

/// System to dynamically enable/disable the Solve Button state (gray out if size is not 3x3x3)
pub fn update_solve_button_state(
    rubik_size: Res<RubikSize>,
    solve_btn_query: Single<(&mut BackgroundColor, &mut BorderColor), With<SolveButton>>,
    mut text_query: Single<&mut Text, With<SolveButtonText>>,
) {
    if rubik_size.is_changed() {
        let (mut bg, mut border) = solve_btn_query.into_inner();
        // Restore beautiful green theme for active solver on all sizes
        *bg = BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.22, 0.15, 0.85)));
        *border = BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.6)));
        text_query.0 = "SOLVE".to_string();
    }
}
