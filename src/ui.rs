use crate::components::{Direction, RotationAxis, RotationMove, RotationQueue};
use bevy::prelude::*;
use rand::RngExt;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, handle_shuffle_button);
    }
}

#[derive(Component)]
struct ShuffleButton;

/// Set up the UI with a premium look
fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        })
        .with_children(|parent| {
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

type InteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor, &'static mut BorderColor),
    (Changed<Interaction>, With<ShuffleButton>),
>;

/// Handle button interaction and trigger shuffle
fn handle_shuffle_button(
    mut interaction_query: InteractionQuery,
    mut rotation_queue: ResMut<RotationQueue>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.3, 0.3, 0.8, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.5, 0.5, 1.0, 1.0)));

                // Trigger shuffle: 20 random moves
                let mut rng = rand::rng();
                for _ in 0..20 {
                    let axis = match rng.random_range(0..3) {
                        0 => RotationAxis::X,
                        1 => RotationAxis::Y,
                        _ => RotationAxis::Z,
                    };
                    let index = rng.random_range(-1..=1);
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
