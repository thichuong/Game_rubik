use crate::ui::components::{EnvControl, EnvList, EnvToggleButton};
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_resvg::prelude::*;
use bevy_resvg::raster::asset::SvgFile;

/// Helper function to spawn the collapsible Environment control accordion
pub fn spawn_environment_section(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    dropdown_icon: &Handle<SvgFile>,
    rotate_left_icon: &Handle<SvgFile>,
    rotate_right_icon: &Handle<SvgFile>,
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
                EnvToggleButton,
            ))
            .with_children(|btn: &mut ChildSpawnerCommands| {
                btn.spawn((
                    Text::new("ENVIRONMENT"),
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

            // Env List Settings
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
                EnvList,
            ))
            .with_children(|list: &mut ChildSpawnerCommands| {
                // Intensity Control
                spawn_intensity_control(list, font);

                // Color Temp presets
                spawn_temp_control(list, font);

                // Light Angle
                spawn_angle_control(list, font, rotate_left_icon, rotate_right_icon);

                // Background Surroundings
                spawn_surroundings_control(list, font);
            });
        });
}

/// Helper function to spawn the Intensity controls
fn spawn_intensity_control(list: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    list.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|group: &mut ChildSpawnerCommands| {
        group.spawn((
            Text::new("Intensity"),
            TextFont {
                font_size: 12.0,
                font: font.clone(),
                ..default()
            },
            TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
        ));

        group
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row: &mut ChildSpawnerCommands| {
                for (label, val) in [("-", -500_000.0), ("+", 500_000.0)] {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(35.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.9))),
                        EnvControl::Intensity(val),
                    ))
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
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

/// Helper function to spawn the Color Temperature controls
fn spawn_temp_control(list: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    list.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|group: &mut ChildSpawnerCommands| {
        group.spawn((
            Text::new("Color Temperature"),
            TextFont {
                font_size: 12.0,
                font: font.clone(),
                ..default()
            },
            TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
        ));

        group
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row: &mut ChildSpawnerCommands| {
                let temps = [
                    (Color::Srgba(Srgba::new(1.0, 0.8, 0.6, 1.0)), "Warm"),
                    (Color::Srgba(Srgba::WHITE), "Neutral"),
                    (Color::Srgba(Srgba::new(0.7, 0.8, 1.0, 1.0)), "Cool"),
                ];
                for (color, label) in temps {
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(color.with_alpha(0.6)),
                        EnvControl::Temp(color),
                    ))
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 11.0,
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

/// Helper function to spawn the Light Angle controls
fn spawn_angle_control(
    list: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    rotate_left_icon: &Handle<SvgFile>,
    rotate_right_icon: &Handle<SvgFile>,
) {
    list.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|group: &mut ChildSpawnerCommands| {
        group.spawn((
            Text::new("Light Angle"),
            TextFont {
                font_size: 12.0,
                font: font.clone(),
                ..default()
            },
            TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.7, 1.0))),
        ));

        group
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row: &mut ChildSpawnerCommands| {
                let angle_controls = [
                    (rotate_left_icon.clone(), -0.5),
                    (rotate_right_icon.clone(), 0.5),
                ];
                for (icon, val) in angle_controls {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(35.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.9))),
                        EnvControl::Angle(val),
                    ))
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            UiSvg(icon),
                            Node {
                                width: Val::Px(14.0),
                                height: Val::Px(14.0),
                                ..default()
                            },
                        ));
                    });
                }
            });
    });
}

/// Helper function to spawn the Surroundings background controls
fn spawn_surroundings_control(list: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    list.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    })
    .with_children(|group: &mut ChildSpawnerCommands| {
        group.spawn((
            Text::new("Surroundings"),
            TextFont {
                font_size: 12.0,
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
            .with_children(|row: &mut ChildSpawnerCommands| {
                let bgs = [
                    (Color::Srgba(Srgba::new(0.05, 0.05, 0.07, 1.0)), "Dark"),
                    (Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 1.0)), "Studio"),
                    (Color::Srgba(Srgba::new(0.1, 0.2, 0.15, 1.0)), "Forest"),
                    (Color::Srgba(Srgba::new(0.3, 0.2, 0.2, 1.0)), "Sunset"),
                ];
                for (color, label) in bgs {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(72.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(color),
                        EnvControl::Bg(color),
                    ))
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 11.0,
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
