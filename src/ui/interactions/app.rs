use super::InteractionQuery;
use crate::ui::components::ExitButton;
use bevy::app::AppExit;
use bevy::prelude::*;

pub fn handle_exit_button(
    mut interaction_query: InteractionQuery<ExitButton>,
    mut app_exit_events: MessageWriter<AppExit>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.1, 0.1, 1.0)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.9, 0.3, 0.3, 1.0)));
                app_exit_events.write(AppExit::Success);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.28, 0.1, 0.1, 0.95)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.8, 0.3, 0.3, 0.85)));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.08, 0.08, 0.85)));
                *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.6, 0.2, 0.2, 0.6)));
            }
        }
    }
}
