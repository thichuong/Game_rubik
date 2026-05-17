mod environment;
mod hud;
mod mapping;
mod sidebar;

use crate::ui::components::{
    ScrollContentWrapper, SidebarScrollHandle, SidebarScrollTrack, SidebarScrollable, SolutionPanel,
};
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;

pub use environment::spawn_environment_section;
pub use hud::spawn_solution_hud;
pub use mapping::spawn_mapping_section;
pub use sidebar::{
    spawn_controls, spawn_divider, spawn_header, spawn_size_section, spawn_skins_section,
};

/// Set up the UI with a premium layout divided into helper functions.
#[allow(clippy::too_many_lines)]
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/font.ttf");
    let dropdown_icon = asset_server.load("textures/icons/dropdown_arrow.svg");
    let rotate_left_icon = asset_server.load("textures/icons/rotate_left.svg");
    let rotate_right_icon = asset_server.load("textures/icons/rotate_right.svg");

    // Unified Left Sidebar Container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                bottom: Val::Px(20.0),
                width: Val::Px(320.0),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::Srgba(Srgba::new(0.06, 0.06, 0.09, 0.85))),
            BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.25, 0.35, 0.4))),
            Interaction::default(),
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            // Scrollable viewport
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::horizontal(Val::Px(20.0)),
                        overflow: Overflow::scroll_y(),
                        border_radius: BorderRadius::all(Val::Px(24.0)),
                        ..default()
                    },
                    Interaction::default(),
                    ScrollPosition::default(),
                    SidebarScrollable,
                ))
                .with_children(|scroll_viewport| {
                    // Scroll Content Wrapper to prevent Flexbox from shrinking items
                    scroll_viewport
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(15.0),
                                padding: UiRect::vertical(Val::Px(20.0)),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ScrollContentWrapper,
                        ))
                        .with_children(|scroll_content: &mut ChildSpawnerCommands| {
                            // Header
                            spawn_header(scroll_content, &font);

                            // Divider
                            spawn_divider(scroll_content);

                            // Cube Size Section
                            spawn_size_section(scroll_content, &font);

                            // Divider
                            spawn_divider(scroll_content);

                            // Controls (Actions)
                            spawn_controls(scroll_content, &font);

                            // Divider
                            spawn_divider(scroll_content);

                            // Cube Skins Accordion
                            spawn_skins_section(scroll_content, &font, &dropdown_icon);

                            // Divider
                            spawn_divider(scroll_content);

                            // Face Mapping Accordion
                            spawn_mapping_section(scroll_content, &font, &dropdown_icon);

                            // Divider
                            spawn_divider(scroll_content);

                            // Environment Settings Accordion
                            spawn_environment_section(
                                scroll_content,
                                &font,
                                &dropdown_icon,
                                &rotate_left_icon,
                                &rotate_right_icon,
                            );
                        });
                });

            // Dynamic Scrollbar Track and Handle (placed as a sibling to the scrollable viewport)
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(4.0),
                        top: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        width: Val::Px(6.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.2))),
                    SidebarScrollTrack,
                ))
                .with_children(|track_parent| {
                    track_parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.25, 0.35, 0.55))),
                        Interaction::default(),
                        SidebarScrollHandle,
                    ));
                });
        });

    // Horizontal Bottom Solution HUD
    commands
        .spawn((
            Node {
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
            },
            BackgroundColor(Color::Srgba(Srgba::new(0.06, 0.1, 0.08, 0.9))),
            BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.4))),
            SolutionPanel,
            Interaction::default(),
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            spawn_solution_hud(parent, &font);
        });
}
