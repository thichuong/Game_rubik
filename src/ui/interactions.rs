use crate::environment::resources::EnvironmentSettings;
use crate::events::{CameraFrameEvent, ResetCameraEvent};
use crate::input::hand_tracking::HandTrackingEnabled;
use crate::rubik::components::{CubieFace, Direction, Face, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{FaceMapping, MoveHistory, RotationQueue, RubikSize, RubikSkin};
use crate::solver::helpers;
use crate::solver::resources::{SolverResource, StepByStepSolution};
use crate::ui::components::{
    CameraFeedImage, CameraTrackingButton, CameraTrackingText, CloseButton, EnvControl, EnvList,
    EnvToggleButton, MappingControl, MappingList, MappingOrderText, MappingToggleButton,
    NextStepButton, ScrollContentWrapper, ShuffleButton, SidebarScrollHandle, SidebarScrollState,
    SidebarScrollable, SizeDecrementButton, SizeIncrementButton, SizeSliderFill, SizeSliderHandle,
    SizeSliderTrack, SizeText, SkinButton, SkinList, SkinToggleButton, SolutionPanel, SolveButton,
    SolveButtonText, StepText,
};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
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
    face_mapping: Res<FaceMapping>,
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
                    let state_str = helpers::get_cube_state(&faces, &cube_query, *face_mapping);

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
                            .map(|m| helpers::move_to_string(m.inverse(), size, *face_mapping))
                            .collect();
                    }
                } else if !history.done.is_empty() {
                    solution.moves = history
                        .done
                        .iter()
                        .rev()
                        .map(|m| helpers::move_to_string(m.inverse(), size, *face_mapping))
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
    face_mapping: Res<FaceMapping>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.5, 0.35, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.6, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let move_str = &solution.moves[solution.current_step];
                    let moves =
                        helpers::solution_to_moves(move_str, rubik_size.size, *face_mapping);
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

pub type MappingToggleQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<MappingToggleButton>),
>;

pub fn handle_mapping_toggle(
    mut interaction_query: MappingToggleQuery,
    mut mapping_list: Single<&mut Node, With<MappingList>>,
    mut state: Local<bool>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *state = !*state;
                mapping_list.display = if *state { Display::Flex } else { Display::None };
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

pub type MappingControlQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static MappingControl,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<Button>),
>;

pub fn handle_mapping_controls(
    mut interaction_query: MappingControlQuery,
    mut mapping: ResMut<FaceMapping>,
) {
    for (interaction, control, mut bg_color) in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) {
            match *control {
                MappingControl::ToggleOrder => {
                    mapping.select_d_first = !mapping.select_d_first;
                }
                MappingControl::SelectF(face) => {
                    if mapping.select_d_first {
                        // D First: only allow if perpendicular to current D
                        if mapping.d_face.normal().dot(face.normal()).abs() < 0.1 {
                            mapping.f_face = face;
                        }
                    } else {
                        // F First: allow any selection, auto-resolve D if conflict
                        mapping.f_face = face;
                        if mapping.d_face.normal().dot(face.normal()).abs() > 0.9 {
                            // Conflict! Auto-select a perpendicular face
                            mapping.d_face = if face == Face::Up || face == Face::Down {
                                Face::Front
                            } else {
                                Face::Up
                            };
                        }
                    }
                }
                MappingControl::SelectD(face) => {
                    if mapping.select_d_first {
                        // D First: allow any selection, auto-resolve F if conflict
                        mapping.d_face = face;
                        if mapping.f_face.normal().dot(face.normal()).abs() > 0.9 {
                            // Conflict! Auto-select a perpendicular face
                            mapping.f_face = if face == Face::Up || face == Face::Down {
                                Face::Front
                            } else {
                                Face::Up
                            };
                        }
                    } else {
                        // F First: only allow if perpendicular to current F
                        if mapping.f_face.normal().dot(face.normal()).abs() < 0.1 {
                            mapping.d_face = face;
                        }
                    }
                }
            }
            *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.5, 1.0)));
        }
    }
}

pub fn update_mapping_ui(
    mapping: Res<FaceMapping>,
    mut order_text_query: Query<&mut Text, With<MappingOrderText>>,
    mut button_query: Query<
        (
            &MappingControl,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    if mapping.is_changed() {
        // 1. Update priority text
        for mut text in &mut order_text_query {
            text.0 = if mapping.select_d_first {
                "Priority: D First".to_string()
            } else {
                "Priority: F First".to_string()
            };
        }

        // 2. Update button styles based on selected FaceMapping
        for (control, mut node, mut bg_color, mut border_color) in &mut button_query {
            match *control {
                MappingControl::ToggleOrder => {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.15, 0.25, 0.85)));
                    *border_color =
                        BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.35, 0.55, 0.6)));
                    node.display = Display::Flex;
                }
                MappingControl::SelectF(face) => {
                    let is_selected = mapping.f_face == face;

                    if mapping.select_d_first {
                        // D First: F is the second choice (only show 4 perpendicular to D)
                        let is_disabled = mapping.d_face.normal().dot(face.normal()).abs() > 0.9;
                        if is_disabled {
                            node.display = Display::None;
                        } else {
                            node.display = Display::Flex;
                            if is_selected {
                                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(
                                    0.15, 0.45, 0.25, 0.85,
                                )));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.8, 0.4, 0.9)));
                            } else {
                                *bg_color =
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                            }
                        }
                    } else {
                        // F First: F is the first choice (show all 6)
                        node.display = Display::Flex;
                        if is_selected {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.45, 0.25, 0.85)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.8, 0.4, 0.9)));
                        } else {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                        }
                    }
                }
                MappingControl::SelectD(face) => {
                    let is_selected = mapping.d_face == face;

                    if mapping.select_d_first {
                        // D First: D is the first choice (show all 6)
                        node.display = Display::Flex;
                        if is_selected {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.45, 0.35, 0.1, 0.85)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.8, 0.6, 0.2, 0.9)));
                        } else {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                        }
                    } else {
                        // F First: D is the second choice (only show 4 perpendicular to F)
                        let is_disabled = mapping.f_face.normal().dot(face.normal()).abs() > 0.9;
                        if is_disabled {
                            node.display = Display::None;
                        } else {
                            node.display = Display::Flex;
                            if is_selected {
                                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(
                                    0.45, 0.35, 0.1, 0.85,
                                )));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.8, 0.6, 0.2, 0.9)));
                            } else {
                                *bg_color =
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_sidebar_scroll(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<
        (
            &bevy::ui::UiGlobalTransform,
            &ComputedNode,
            &mut ScrollPosition,
        ),
        With<SidebarScrollable>,
    >,
    content_query: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    viewport_query: Option<Single<&ComputedNode, With<SidebarScrollable>>>,
    window_query: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    let Some(window) = window_query else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let mut scroll_dy = 0.0;
    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 35.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };
        scroll_dy -= dy;
    }

    if scroll_dy != 0.0 {
        let content_height = if let Some(node) = content_query {
            node.size.y
        } else {
            0.0
        };
        let viewport_height = if let Some(node) = viewport_query {
            node.size.y
        } else {
            0.0
        };
        let max_scroll = (content_height - viewport_height).max(0.0);

        for (transform, computed_node, mut scroll_pos) in &mut query {
            let size = computed_node.size();
            let center_x = transform.translation.x;
            let center_y = transform.translation.y;
            let half_w = size.x / 2.0;
            let half_h = size.y / 2.0;

            let is_hovered = cursor_pos.x >= center_x - half_w
                && cursor_pos.x <= center_x + half_w
                && cursor_pos.y >= center_y - half_h
                && cursor_pos.y <= center_y + half_h;

            if is_hovered {
                scroll_pos.0.y = (scroll_pos.0.y + scroll_dy).clamp(0.0, max_scroll);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_sidebar_scrollbar_visuals(
    scroll_data: Option<Single<(&ComputedNode, &ScrollPosition), With<SidebarScrollable>>>,
    content_node: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    handle_data: Option<
        Single<(&Interaction, &mut Node, &mut BackgroundColor), With<SidebarScrollHandle>>,
    >,
    scroll_state: Res<SidebarScrollState>,
) {
    let Some(scroll_data) = scroll_data else {
        return;
    };
    let Some(content_node) = content_node else {
        return;
    };
    let Some(mut handle_data) = handle_data else {
        return;
    };

    let viewport_height = scroll_data.0.size.y;
    let scroll_pos = scroll_data.1;
    let content_height = content_node.size.y;

    if content_height <= viewport_height {
        handle_data.1.display = Display::None;
        return;
    }

    handle_data.1.display = Display::Flex;

    let track_height = viewport_height - 20.0; // 10px top/bottom padding
    let handle_height = ((viewport_height / content_height) * track_height).max(30.0);

    let max_scroll = content_height - viewport_height;
    let ratio = if max_scroll > 0.0 {
        scroll_pos.0.y / max_scroll
    } else {
        0.0
    };
    let handle_top = ratio * (track_height - handle_height);

    handle_data.1.height = Val::Px(handle_height);
    handle_data.1.top = Val::Px(handle_top);

    let interaction = handle_data.0;

    // Dynamic background color based on interaction state
    if scroll_state.is_dragging {
        *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.6, 0.9, 0.9)));
    } else {
        match *interaction {
            Interaction::Pressed => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.6, 0.9, 0.9)));
            }
            Interaction::Hovered => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.35, 0.35, 0.45, 0.8)));
            }
            Interaction::None => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.25, 0.35, 0.55)));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_sidebar_scrollbar_drag(
    mut scroll_state: ResMut<SidebarScrollState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Option<Single<&Window, With<PrimaryWindow>>>,
    scroll_data: Option<Single<(&ComputedNode, &mut ScrollPosition), With<SidebarScrollable>>>,
    content_node: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    handle_data: Option<Single<(&Interaction, &ComputedNode), With<SidebarScrollHandle>>>,
) {
    let Some(mut scroll_data) = scroll_data else {
        return;
    };
    let Some(content_node) = content_node else {
        return;
    };
    let Some(handle_data) = handle_data else {
        return;
    };

    let Some(window) = windows else {
        return;
    };
    let cursor_y = if let Some(pos) = window.cursor_position() {
        pos.y
    } else {
        return;
    };

    let viewport_height = scroll_data.0.size.y;
    let content_height = content_node.size.y;
    let handle_height = handle_data.1.size.y;

    let max_scroll = (content_height - viewport_height).max(0.0);
    if max_scroll <= 0.0 {
        scroll_state.is_dragging = false;
        return;
    }

    let track_height = viewport_height - 20.0;
    let scrollable_track_range = (track_height - handle_height).max(1.0);

    let interaction = handle_data.0;

    if mouse_input.just_pressed(MouseButton::Left) && *interaction == Interaction::Pressed {
        scroll_state.is_dragging = true;
        scroll_state.drag_start_cursor_y = cursor_y;
        scroll_state.drag_start_scroll_y = scroll_data.1 .0.y;
    }

    if scroll_state.is_dragging {
        if mouse_input.pressed(MouseButton::Left) {
            let delta_cursor_y = cursor_y - scroll_state.drag_start_cursor_y;
            let delta_scroll_y = delta_cursor_y * (max_scroll / scrollable_track_range);
            scroll_data.1 .0.y =
                (scroll_state.drag_start_scroll_y + delta_scroll_y).clamp(0.0, max_scroll);
        } else {
            scroll_state.is_dragging = false;
        }
    }
}

pub type CameraToggleQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<Interaction>, With<CameraTrackingButton>),
>;

pub fn handle_camera_toggle(
    mut interaction_query: CameraToggleQuery,
    mut text_query: Single<&mut Text, With<CameraTrackingText>>,
    mut image_node: Option<Single<&mut Node, With<CameraFeedImage>>>,
    mut enabled: ResMut<HandTrackingEnabled>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                enabled.0 = !enabled.0;
                if enabled.0 {
                    text_query.0 = "CAMERA: ON".to_string();
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.6, 0.3, 0.85)));
                    *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.5, 0.9)));
                    if let Some(ref mut img) = image_node {
                        img.display = Display::Flex;
                    }
                } else {
                    text_query.0 = "CAMERA: OFF".to_string();
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.8)));
                    *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.3, 0.4, 0.5)));
                    if let Some(ref mut img) = image_node {
                        img.display = Display::None;
                    }
                }
            }
            Interaction::Hovered => {
                if !enabled.0 {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.9)));
                }
            }
            Interaction::None => {
                if !enabled.0 {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.8)));
                }
            }
        }
    }
}

pub fn update_camera_feed(
    mut events: MessageReader<CameraFrameEvent>,
    mut images: ResMut<Assets<Image>>,
    mut image_node: Option<Single<&mut ImageNode, With<CameraFeedImage>>>,
) {
    // Only process the latest frame if multiple arrived
    if let Some(latest_event) = events.read().last() {
        if let Some(ref mut img_node) = image_node {
            let image = Image::new(
                Extent3d {
                    width: latest_event.width,
                    height: latest_event.height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                latest_event.frame_rgba.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );

            let handle = images.add(image);
            img_node.image = handle;
        }
    }
}
