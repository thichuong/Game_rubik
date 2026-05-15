use crate::components::{CubieFace, Direction, RotationAxis, RotationMove, RotationQueue};
use crate::solver;
use bevy::prelude::*;
use rand::RngExt;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (handle_shuffle_button, handle_solve_button));
    }
}

#[derive(Component)]
struct ShuffleButton;

#[derive(Component)]
struct SolveButton;

/// Set up the UI with a premium look
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
            // SOLVE Button
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

/// Handle solve button interaction
fn handle_solve_button(
    mut interaction_query: InteractionQuery<SolveButton>,
    mut rotation_queue: ResMut<RotationQueue>,
    faces: Query<(&CubieFace, &GlobalTransform)>,
    solver_res: Res<solver::SolverResource>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.8, 0.3, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 1.0, 0.5, 1.0)));

                let state_str = solver::get_cube_state(&faces);
                info!("Cube state for solver: {}", state_str);

                match kewb::FaceCube::try_from(state_str.as_str()) {
                    Ok(face_cube) => match kewb::CubieCube::try_from(&face_cube) {
                        Ok(cubie_cube) => {
                            let mut solver = kewb::Solver::new(&solver_res.table, 23, None);
                            if let Some(solution) = solver.solve(cubie_cube) {
                                info!("Solution found: {}", solution);
                                let moves = solver::solution_to_moves(&solution.to_string());
                                for m in moves {
                                    rotation_queue.0.push_back(m);
                                }
                            } else {
                                warn!("No solution found for this state.");
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
