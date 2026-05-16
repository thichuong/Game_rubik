use crate::components::{
    CubieFace, Direction, RotationAxis, RotationMove, RotationQueue, RubikSkin, SkinType,
};
use crate::solver;
use bevy::prelude::*;
use rand::RngExt;
use std::fmt::Write;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (
                handle_shuffle_button,
                handle_solve_button,
                handle_next_step_button,
                handle_close_button,
                update_solution_panel,
                handle_skin_button,
            ),
        );
    }
}

#[derive(Component)]
struct ShuffleButton;

#[derive(Component)]
struct SolveButton;

#[derive(Component)]
struct NextStepButton;

#[derive(Component)]
struct SolutionPanel;

#[derive(Component)]
struct StepText;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
struct SkinButton(SkinType);

/// Set up the UI with a premium look
#[allow(clippy::too_many_lines)]
fn setup_ui(mut commands: Commands) {
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
            // SOLVE Button (Triggers Step-by-Step)
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

            // SKINS Panel (Above the buttons)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SKINS"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::new(0.8, 0.8, 0.9, 1.0))),
                    ));

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            let skins = [
                                (SkinType::Classic, "Classic"),
                                (SkinType::Carbon, "Carbon"),
                                (SkinType::Geometric, "Geo"),
                                (SkinType::Floral, "Floral"),
                            ];

                            for (skin_type, label) in skins {
                                parent
                                    .spawn(Button)
                                    .insert(Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(40.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        ..default()
                                    })
                                    .insert(BackgroundColor(Color::Srgba(Srgba::new(
                                        0.2, 0.2, 0.25, 0.8,
                                    ))))
                                    .insert(SkinButton(skin_type))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(Color::Srgba(Srgba::WHITE)),
                                        ));
                                    });
                            }
                        });
                });
        });

    // Solution Panel (Top left)
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
            // Close Button
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

            // Next Step Button
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

            // Face Legend (Added back as labels on Rubik are disabled)
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

type InteractionQuery<'w, 's, T> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<Interaction>, With<T>),
>;

/// Handle shuffle button interaction
fn handle_shuffle_button(
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

/// Handle solve button interaction (enters step-by-step mode)
fn handle_solve_button(
    mut interaction_query: InteractionQuery<SolveButton>,
    mut solution: ResMut<crate::components::StepByStepSolution>,
    mut reset_camera: MessageWriter<crate::components::ResetCameraEvent>,
    faces: Query<(&CubieFace, &GlobalTransform)>,
    solver_res: Res<solver::SolverResource>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.8, 0.3, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 1.0, 0.5, 1.0)));

                // Trigger camera reset and labels
                reset_camera.write(crate::components::ResetCameraEvent);

                // Set active immediately to show labels and panel
                solution.active = true;
                solution.moves.clear();
                solution.current_step = 0;

                let state_str = solver::get_cube_state(&faces);

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

/// Handle next step button interaction
fn handle_next_step_button(
    mut interaction_query: InteractionQuery<NextStepButton>,
    mut solution: ResMut<crate::components::StepByStepSolution>,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    for (interaction, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.8, 0.4, 1.0)));

                if solution.active && solution.current_step < solution.moves.len() {
                    let move_str = &solution.moves[solution.current_step];
                    let moves = solver::solution_to_moves(move_str);
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

/// Update the visibility and content of the solution panel
fn update_solution_panel(
    solution: Res<crate::components::StepByStepSolution>,
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

/// Handle close button interaction
fn handle_close_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
    mut solution: ResMut<crate::components::StepByStepSolution>,
) {
    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) {
            solution.active = false;
        }
    }
}

/// Handle skin button interaction
fn handle_skin_button(
    mut interaction_query: Query<
        (&Interaction, &SkinButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut rubik_skin: ResMut<RubikSkin>,
) {
    for (interaction, skin_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                rubik_skin.current = skin_btn.0;
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.8, 1.0)));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.3, 0.4, 0.9)));
            }
            Interaction::None => {
                if rubik_skin.current == skin_btn.0 {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.3, 0.6, 1.0)));
                } else {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.8)));
                }
            }
        }
    }
}
