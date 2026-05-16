use crate::events::ResetCameraEvent;
use crate::rubik::components::{CubieFace, Direction, RotationAxis, RotationMove, RubikCube};
use crate::rubik::resources::{RotationQueue, RubikSkin, SkinType};
use crate::solver::helpers;
use crate::solver::resources::{SolverResource, StepByStepSolution};
use bevy::prelude::*;
use rand::RngExt;
use std::fmt::Write;

#[derive(Component)]
pub struct ShuffleButton;

#[derive(Component)]
pub struct SolveButton;

#[derive(Component)]
pub struct NextStepButton;

#[derive(Component)]
pub struct SolutionPanel;

#[derive(Component)]
pub struct StepText;

#[derive(Component)]
pub struct CloseButton;

#[derive(Component)]
pub struct SkinButton(pub SkinType);

#[derive(Component)]
pub struct SkinToggleButton;

#[derive(Component)]
pub struct SkinList;

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

/// Set up the UI with a premium look
#[allow(clippy::too_many_lines)]
pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(40.0)),
                column_gap: Val::Px(20.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // SOLVE Button
            parent
                .spawn(Button)
                .insert(Node {
                    width: Val::Px(160.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(15.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::Srgba(Srgba::new(
                    0.3, 0.4, 0.3, 1.0,
                ))))
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.15, 0.2, 0.15, 0.8,
                ))))
                .insert(SolveButton)
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SOLVE"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });

            // SHUFFLE Button
            parent
                .spawn(Button)
                .insert(Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(15.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::Srgba(Srgba::new(
                    0.3, 0.3, 0.4, 1.0,
                ))))
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.15, 0.15, 0.2, 0.8,
                ))))
                .insert(ShuffleButton)
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SHUFFLE"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });
        });

    // TOP RIGHT Skin Panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                right: Val::Px(40.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(10.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // Toggle Button
            parent
                .spawn(Button)
                .insert(Node {
                    width: Val::Px(160.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.15, 0.15, 0.25, 0.9,
                ))))
                .insert(SkinToggleButton)
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SKINS ▾"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });

            // Skin List
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        display: Display::None,
                        ..default()
                    },
                    SkinList,
                ))
                .with_children(|parent| {
                    let skins = [
                        (SkinType::Classic, "Classic"),
                        (SkinType::Carbon, "Carbon Fiber"),
                        (SkinType::Geometric, "Geometric"),
                        (SkinType::Floral, "Floral Pattern"),
                    ];

                    for (skin_type, label) in skins {
                        parent
                            .spawn(Button)
                            .insert(Node {
                                width: Val::Px(160.0),
                                height: Val::Px(45.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::Srgba(Srgba::new(
                                0.2, 0.2, 0.2, 0.8,
                            ))))
                            .insert(SkinButton(skin_type))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::Srgba(Srgba::WHITE)),
                                ));
                            });
                    }
                });
        });

    // Solution Panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: Val::Px(40.0),
                width: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(15.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 0.9))),
            BorderColor::all(Color::Srgba(Srgba::new(0.5, 0.5, 0.5, 1.0))),
            SolutionPanel,
        ))
        .with_children(|parent| {
            parent
                .spawn(Button)
                .insert(Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                })
                .insert(CloseButton);

            parent.spawn((
                Text::new("SOLUTION STEPS"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::WHITE)),
            ));

            parent.spawn((
                Text::new("No solution yet"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::new(0.7, 0.7, 0.7, 1.0))),
                StepText,
                Node {
                    margin: UiRect::vertical(Val::Px(15.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Button)
                .insert(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.2, 0.4, 0.2, 1.0,
                ))))
                .insert(NextStepButton)
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("NEXT STEP"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });

            // Face Legend
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    let legend = [
                        ("U", "Up (White center)"),
                        ("D", "Down (Yellow center)"),
                        ("F", "Front (Green center)"),
                        ("B", "Back (Blue center)"),
                        ("L", "Left (Orange center)"),
                        ("R", "Right (Red center)"),
                    ];

                    for (abbr, desc) in legend {
                        parent.spawn((
                            Text::new(format!("{abbr}: {desc}")),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
                        ));
                    }
                });
        });
}

pub fn handle_shuffle_button(
    mut interaction_query: InteractionQuery<ShuffleButton>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.3, 0.8, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 0.5, 1.0, 1.0)));

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
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.2, 0.3, 0.9)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.4, 0.6, 1.0)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.8)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.3, 0.4, 1.0)));
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
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.8, 0.3, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 1.0, 0.5, 1.0)));

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
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.3, 0.2, 0.9)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.6, 0.4, 1.0)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.2, 0.15, 0.8)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.4, 0.3, 1.0)));
            }
        }
    }
}

pub fn handle_next_step_button(
    mut interaction_query: InteractionQuery<NextStepButton>,
    mut solution: ResMut<StepByStepSolution>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    for (interaction, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.8, 0.4, 1.0)));

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
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.5, 0.3, 0.9)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.4, 0.2, 1.0)));
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
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.3, 0.5, 1.0)));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.25, 0.4, 0.95)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.25, 0.9)));
            }
        }
    }
}
