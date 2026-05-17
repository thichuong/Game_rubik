mod environment;
mod hud;
mod sidebar;

use crate::ui::components::SolutionPanel;
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;

pub use environment::spawn_environment_section;
pub use hud::spawn_solution_hud;
pub use sidebar::{
    spawn_controls, spawn_divider, spawn_header, spawn_size_section, spawn_skins_section,
};

/// Set up the UI with a premium layout divided into helper functions.
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/font.ttf");
    let dropdown_icon = asset_server.load("textures/icons/dropdown_arrow.svg");
    let rotate_left_icon = asset_server.load("textures/icons/rotate_left.svg");
    let rotate_right_icon = asset_server.load("textures/icons/rotate_right.svg");

    // Unified Left Sidebar
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(320.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(15.0),
            border_radius: BorderRadius::all(Val::Px(24.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::Srgba(Srgba::new(
            0.06, 0.06, 0.09, 0.85,
        ))))
        .insert(BorderColor::all(Color::Srgba(Srgba::new(
            0.25, 0.25, 0.35, 0.4,
        ))))
        .insert(Pickable::IGNORE)
        .with_children(|parent: &mut ChildSpawnerCommands| {
            // Header
            spawn_header(parent, &font);

            // Divider
            spawn_divider(parent);

            // Cube Size Section
            spawn_size_section(parent, &font);

            // Divider
            spawn_divider(parent);

            // Controls (Actions)
            spawn_controls(parent, &font);

            // Divider
            spawn_divider(parent);

            // Cube Skins Accordion
            spawn_skins_section(parent, &font, &dropdown_icon);

            // Divider
            spawn_divider(parent);

            // Environment Settings Accordion
            spawn_environment_section(
                parent,
                &font,
                &dropdown_icon,
                &rotate_left_icon,
                &rotate_right_icon,
            );
        });

    // Horizontal Bottom Solution HUD
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(360.0),
            right: Val::Px(40.0),
            height: Val::Px(150.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(10.0),
            display: Display::None,
            border_radius: BorderRadius::all(Val::Px(20.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::Srgba(Srgba::new(
            0.06, 0.1, 0.08, 0.9,
        ))))
        .insert(BorderColor::all(Color::Srgba(Srgba::new(
            0.2, 0.5, 0.3, 0.4,
        ))))
        .insert(SolutionPanel)
        .with_children(|parent: &mut ChildSpawnerCommands| {
            spawn_solution_hud(parent, &font);
        });
}
