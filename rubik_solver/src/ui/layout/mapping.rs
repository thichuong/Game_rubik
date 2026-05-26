use crate::rubik::components::Face;
use crate::ui::components::{MappingControl, MappingList, MappingOrderText, MappingToggleButton};
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_resvg::prelude::*;
use bevy_resvg::raster::asset::SvgFile;

/// Helper function to spawn the collapsible Face Mapping customizer accordion
#[allow(clippy::too_many_lines)]
pub fn spawn_mapping_section(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    dropdown_icon: &Handle<SvgFile>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            // Toggle Header Button
            p.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.25, 0.3, 0.4))),
                BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.12, 0.15, 0.6))),
                MappingToggleButton,
            ))
            .with_children(|btn: &mut ChildSpawnerCommands| {
                btn.spawn((
                    Text::new("FACE MAPPING"),
                    TextFont {
                        font_size: 14.0,
                        font: font.clone(),
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::WHITE)),
                ));
                btn.spawn((
                    UiSvg(dropdown_icon.clone()),
                    Node {
                        width: Val::Px(10.0),
                        height: Val::Px(10.0),
                        ..default()
                    },
                ));
            });

            // Mapping List
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    display: Display::None,
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    width: Val::Percent(100.0),
                    ..default()
                },
                MappingList,
            ))
            .with_children(|list: &mut ChildSpawnerCommands| {
                // 1. Order Toggle Button
                list.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(34.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.5)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.3, 0.5, 0.6))),
                    BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.15, 0.25, 0.85))),
                    MappingControl::ToggleOrder,
                ))
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("Priority: F First"),
                        TextFont {
                            font_size: 13.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                        MappingOrderText,
                    ));
                });

                // 2. Select F Section
                list.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|group: &mut ChildSpawnerCommands| {
                    group.spawn((
                        Text::new("SELECT FRONT FACE (F)"),
                        TextFont {
                            font_size: 11.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
                    ));

                    group
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|grid: &mut ChildSpawnerCommands| {
                            let faces = [
                                (Face::Up, "U (White)", Color::Srgba(Srgba::WHITE)),
                                (
                                    Face::Down,
                                    "D (Yellow)",
                                    Color::Srgba(Srgba::new(1.0, 0.9, 0.0, 1.0)),
                                ),
                                (
                                    Face::Left,
                                    "L (Orange)",
                                    Color::Srgba(Srgba::new(1.0, 0.4, 0.0, 1.0)),
                                ),
                                (
                                    Face::Right,
                                    "R (Red)",
                                    Color::Srgba(Srgba::new(0.9, 0.1, 0.1, 1.0)),
                                ),
                                (
                                    Face::Front,
                                    "F (Green)",
                                    Color::Srgba(Srgba::new(0.1, 0.7, 0.1, 1.0)),
                                ),
                                (
                                    Face::Back,
                                    "B (Blue)",
                                    Color::Srgba(Srgba::new(0.1, 0.2, 0.9, 1.0)),
                                ),
                            ];

                            for (face, label, bg_col) in faces {
                                grid.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(126.0),
                                        height: Val::Px(28.0),
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::left(Val::Px(8.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4))),
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7))),
                                    MappingControl::SelectF(face),
                                ))
                                .with_children(
                                    |btn: &mut ChildSpawnerCommands| {
                                        // Mini color indicator
                                        btn.spawn((
                                            Node {
                                                width: Val::Px(10.0),
                                                height: Val::Px(10.0),
                                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                                margin: UiRect::right(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_col),
                                        ));

                                        btn.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font_size: 11.0,
                                                font: font.clone(),
                                                ..default()
                                            },
                                            TextColor(Color::Srgba(Srgba::WHITE)),
                                        ));
                                    },
                                );
                            }
                        });
                });

                // 3. Select D Section
                list.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|group: &mut ChildSpawnerCommands| {
                    group.spawn((
                        Text::new("SELECT DOWN FACE (D)"),
                        TextFont {
                            font_size: 11.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
                    ));

                    group
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|grid: &mut ChildSpawnerCommands| {
                            let faces = [
                                (Face::Up, "U (White)", Color::Srgba(Srgba::WHITE)),
                                (
                                    Face::Down,
                                    "D (Yellow)",
                                    Color::Srgba(Srgba::new(1.0, 0.9, 0.0, 1.0)),
                                ),
                                (
                                    Face::Left,
                                    "L (Orange)",
                                    Color::Srgba(Srgba::new(1.0, 0.4, 0.0, 1.0)),
                                ),
                                (
                                    Face::Right,
                                    "R (Red)",
                                    Color::Srgba(Srgba::new(0.9, 0.1, 0.1, 1.0)),
                                ),
                                (
                                    Face::Front,
                                    "F (Green)",
                                    Color::Srgba(Srgba::new(0.1, 0.7, 0.1, 1.0)),
                                ),
                                (
                                    Face::Back,
                                    "B (Blue)",
                                    Color::Srgba(Srgba::new(0.1, 0.2, 0.9, 1.0)),
                                ),
                            ];

                            for (face, label, bg_col) in faces {
                                grid.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(126.0),
                                        height: Val::Px(28.0),
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::left(Val::Px(8.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4))),
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7))),
                                    MappingControl::SelectD(face),
                                ))
                                .with_children(
                                    |btn: &mut ChildSpawnerCommands| {
                                        // Mini color indicator
                                        btn.spawn((
                                            Node {
                                                width: Val::Px(10.0),
                                                height: Val::Px(10.0),
                                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                                margin: UiRect::right(Val::Px(8.0)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_col),
                                        ));

                                        btn.spawn((
                                            Text::new(label),
                                            TextFont {
                                                font_size: 11.0,
                                                font: font.clone(),
                                                ..default()
                                            },
                                            TextColor(Color::Srgba(Srgba::WHITE)),
                                        ));
                                    },
                                );
                            }
                        });
                });
            });
        });
}
