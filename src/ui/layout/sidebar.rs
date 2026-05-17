use crate::rubik::resources::SkinType;
use crate::ui::components::{
    ShuffleButton, SizeDecrementButton, SizeIncrementButton, SizeSliderFill, SizeSliderHandle,
    SizeSliderTrack, SizeText, SkinButton, SkinList, SkinToggleButton, SolveButton,
    SolveButtonText,
};
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
                row.spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(1.5)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.3, 0.5, 0.6))),
                    BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.15, 0.25, 0.85))),
                    ShuffleButton,
                ))
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
                row.spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(1.5)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.5, 0.3, 0.6))),
                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.22, 0.15, 0.85))),
                    SolveButton,
                ))
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("SOLVE"),
                        TextFont {
                            font_size: 15.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                        SolveButtonText,
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
                SkinToggleButton,
            ))
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
                    grid.spawn((
                        Button,
                        Node {
                            width: Val::Px(136.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4))),
                        BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7))),
                        SkinButton(skin_type),
                    ))
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

/// Helper function to spawn Cube Size controller (with premium slider and - / + buttons)
#[allow(clippy::too_many_lines)]
pub fn spawn_size_section(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            // Label displaying current Rubik size
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header: &mut ChildSpawnerCommands| {
                header.spawn((
                    Text::new("CUBE SIZE"),
                    TextFont {
                        font_size: 12.0,
                        font: font.clone(),
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
                ));
                header.spawn((
                    Text::new("3x3x3"),
                    TextFont {
                        font_size: 13.0,
                        font: font.clone(),
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::new(0.4, 0.7, 1.0, 1.0))),
                    SizeText,
                ));
            });

            // Slider control container (Layout: [ - ]  ====O====  [ + ])
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                ..default()
            })
            .with_children(|row: &mut ChildSpawnerCommands| {
                // Decrement button
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.12, 0.18, 0.8))),
                    SizeDecrementButton,
                ))
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("-"),
                        TextFont {
                            font_size: 18.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });

                // Slider Container (Holds visuals and the invisible hit box)
                row.spawn(Node {
                    flex_grow: 1.0,
                    height: Val::Px(30.0), // Large hit box container
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|container: &mut ChildSpawnerCommands| {
                    // 1. Visual Track (Bottom layer)
                    container.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(8.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            position_type: PositionType::Absolute,
                            top: Val::Px(11.0),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.08, 0.08, 0.12, 1.0))),
                    ));

                    // 2. Slider Fill (Middle layer)
                    container.spawn((
                        Node {
                            width: Val::Percent(10.0), // Updated by system
                            height: Val::Px(8.0),
                            border_radius: BorderRadius::new(
                                Val::Px(4.0),
                                Val::Px(0.0),
                                Val::Px(0.0),
                                Val::Px(4.0),
                            ),
                            position_type: PositionType::Absolute,
                            top: Val::Px(11.0),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.5, 0.9, 0.85))),
                        SizeSliderFill,
                    ));

                    // 3. Slider Handle (Top visual layer)
                    container.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(5.0),
                            left: Val::Percent(10.0), // Updated by system
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            margin: UiRect::left(Val::Px(-10.0)), // Offset to center thumb
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::WHITE)),
                        BorderColor::all(Color::Srgba(Srgba::new(0.15, 0.45, 0.85, 1.0))),
                        SizeSliderHandle,
                    ));

                    // 4. INVISIBLE OVERLAY HIT BOX (Captures ALL clicks!)
                    container.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.01))), // 1% opacity to ensure picking
                        SizeSliderTrack, // The track system will query THIS!
                    ));
                });

                // Increment button
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.12, 0.18, 0.8))),
                    SizeIncrementButton,
                ))
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 18.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });
            });
        });
}
