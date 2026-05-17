use crate::environment::resources::EnvironmentSettings;
use crate::events::ResetCameraEvent;
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{RotationQueue, RubikSkin};
use crate::solver::helpers;
use crate::solver::resources::{SolverResource, StepByStepSolution};
use crate::ui::components::{
    CloseButton, EnvControl, EnvList, EnvToggleButton, NextStepButton, ShuffleButton, SkinButton,
    SkinList, SkinToggleButton, SolutionPanel, SolveButton, StepText,
};
use bevy::prelude::*;
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
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.3, 0.55, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 0.6, 0.9, 1.0)));

                let mut rng = rand::rng();
                for _ in 0..20 {
                    let axis = match rng.random_range(0..3) {
                        0 => RotationAxis::X,
                        1 => RotationAxis::Y,
                        _ => RotationAxis::Z,
                    };
                    let index = if rng.random_bool(0.5) { -1 } else { 1 };
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
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.5, 0.35, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                reset_camera.write(ResetCameraEvent);

                solution.active = true;
                solution.moves.clear();
                solution.current_step = 0;

                let state_str = helpers::get_cube_state(&faces, &cube_query);

                match kewb::FaceCube::try_from(state_str.as_str()) {
                    Ok(face_cube) => match kewb::CubieCube::try_from(&face_cube) {
                        Ok(cubie_cube) => {
                            let mut solver = kewb::Solver::new(&solver_res.table, 23, None);
                            if let Some(sol) = solver.solve(cubie_cube) {
                                solution.moves = sol
                                    .to_string()
                                    .split_whitespace()
                                    .map(String::from)
                                    .collect();
                            }
                        }
                        Err(e) => error!("Invalid cube state: {:?}", e),
                    },
                    Err(e) => error!("Failed to parse face cube: {:?}", e),
                }
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
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.5, 0.35, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let move_str = &solution.moves[solution.current_step];
                    let moves = helpers::solution_to_moves(move_str);
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
            let mut full_text = String::new();
            for (i, m) in solution.moves.iter().enumerate() {
                if i == solution.current_step {
                    let _ = write!(full_text, " >>{m}<< ");
                } else {
                    let _ = write!(full_text, " {m} ");
                }
            }

            if solution.current_step >= solution.moves.len() {
                text.0 = "Solved!".to_string();
            } else {
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
