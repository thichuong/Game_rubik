use crate::rubik::resources::SkinType;
use crate::ui::components::{ShuffleButton, SkinButton, SkinList, SkinToggleButton, SolveButton};
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_resvg::prelude::*;
use bevy_resvg::raster::asset::SvgFile;

/// Helper function to spawn the Studio Header
pub fn spawn_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(2.0),
            margin: UiRect::bottom(Val::Px(5.0)),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            p.spawn((
                Text::new("RUBIK STUDIO"),
                TextFont {
                    font_size: 26.0,
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::WHITE)),
            ));
            p.spawn((
                Text::new("MODULAR SOLVER & CUSTOMIZER"),
                TextFont {
                    font_size: 10.0,
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::new(0.5, 0.5, 0.6, 1.0))),
            ));
        });
}

/// Helper function to spawn visual separation dividers
pub fn spawn_divider(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.5),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.2, 0.3, 0.4))),
    ));
}

/// Helper function to spawn Shuffle and Solve control actions
pub fn spawn_controls(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            p.spawn((
                Text::new("CONTROLS"),
                TextFont {
                    font_size: 12.0,
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
            ));

            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|row: &mut ChildSpawnerCommands| {
                // SHUFFLE Button
                row.spawn(Button)
                    .insert(Node {
                        flex_grow: 1.0,
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(1.5)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(Color::Srgba(Srgba::new(
                        0.25, 0.3, 0.5, 0.6,
                    ))))
                    .insert(BackgroundColor(Color::Srgba(Srgba::new(
                        0.12, 0.15, 0.25, 0.85,
                    ))))
                    .insert(ShuffleButton)
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new("SHUFFLE"),
                            TextFont {
                                font_size: 15.0,
                                font: font.clone(),
                                ..default()
                            },
                            TextColor(Color::Srgba(Srgba::WHITE)),
                        ));
                    });

                // SOLVE Button
                row.spawn(Button)
                    .insert(Node {
                        flex_grow: 1.0,
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(1.5)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(Color::Srgba(Srgba::new(
                        0.2, 0.5, 0.3, 0.6,
                    ))))
                    .insert(BackgroundColor(Color::Srgba(Srgba::new(
                        0.1, 0.22, 0.15, 0.85,
                    ))))
                    .insert(SolveButton)
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new("SOLVE"),
                            TextFont {
                                font_size: 15.0,
                                font: font.clone(),
                                ..default()
                            },
                            TextColor(Color::Srgba(Srgba::WHITE)),
                        ));
                    });
            });
        });
}

/// Helper function to spawn the collapsible Cube Skins grid
pub fn spawn_skins_section(
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
            p.spawn(Button)
                .insert(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::Srgba(Srgba::new(
                    0.25, 0.25, 0.3, 0.4,
                ))))
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.12, 0.12, 0.15, 0.6,
                ))))
                .insert(SkinToggleButton)
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("CUBE SKINS"),
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

            // Skin List Grid
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    column_gap: Val::Px(8.0),
                    display: Display::None,
                    width: Val::Percent(100.0),
                    ..default()
                },
                SkinList,
            ))
            .with_children(|grid: &mut ChildSpawnerCommands| {
                let skins = [
                    (SkinType::Classic, "Classic"),
                    (SkinType::Carbon, "Carbon Fiber"),
                    (SkinType::Geometric, "Geometric"),
                    (SkinType::Floral, "Floral Pattern"),
                ];

                for (skin_type, label) in skins {
                    grid.spawn(Button)
                        .insert(Node {
                            width: Val::Px(136.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        })
                        .insert(BorderColor::all(Color::Srgba(Srgba::new(
                            0.2, 0.2, 0.25, 0.4,
                        ))))
                        .insert(BackgroundColor(Color::Srgba(Srgba::new(
                            0.1, 0.1, 0.12, 0.7,
                        ))))
                        .insert(SkinButton(skin_type))
                        .with_children(|btn: &mut ChildSpawnerCommands| {
                            btn.spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: 13.0,
                                    font: font.clone(),
                                    ..default()
                                },
                                TextColor(Color::Srgba(Srgba::WHITE)),
                            ));
                        });
                }
            });
        });
}
