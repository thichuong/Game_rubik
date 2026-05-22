use crate::environment::resources::EnvironmentSettings;
use crate::ui::components::{EnvControl, EnvList, EnvToggleButton};
use bevy::prelude::*;

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
