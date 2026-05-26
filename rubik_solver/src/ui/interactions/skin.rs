use crate::rubik::resources::RubikSkin;
use crate::ui::components::{SkinButton, SkinList, SkinToggleButton};
use bevy::prelude::*;

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
