use crate::ui::components::{CloseButton, NextStepButton, StepText};
use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;

/// Helper function to spawn the Bottom Solution steps HUD content
pub fn spawn_solution_hud(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    // Row 1: Header & Close button
    spawn_hud_header(parent, font);

    // Row 2: Solution details & Next button
    spawn_hud_details_row(parent, font);
}

/// Helper function to spawn the HUD's top row (Header and Close button)
fn spawn_hud_header(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            p.spawn((
                Text::new("SOLUTION STEPS"),
                TextFont {
                    font_size: 16.0,
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::Srgba(Srgba::new(0.4, 0.8, 0.5, 1.0))),
            ));

            p.spawn(Button)
                .insert(Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .insert(CloseButton)
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 14.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::new(0.6, 0.6, 0.6, 1.0))),
                    ));
                });
        });
}

/// Helper function to spawn the HUD's bottom row (Steps display, legend and action button)
fn spawn_hud_details_row(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            column_gap: Val::Px(15.0),
            ..default()
        })
        .with_children(|p: &mut ChildSpawnerCommands| {
            // Left Column: Step display & Legend
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|col: &mut ChildSpawnerCommands| {
                col.spawn((
                    Text::new("No solution yet"),
                    TextFont {
                        font_size: 16.0,
                        font: font.clone(),
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::WHITE)),
                    StepText,
                ));

                col.spawn((
                    Text::new("U: Up | D: Down | F: Front | B: Back | L: Left | R: Right\nX/Y/Z [idx]: Inner slice 'idx' (X: Left-Right | Y: Up-Down | Z: Front-Back)"),
                    TextFont {
                        font_size: 10.5,
                        font: font.clone(),
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::new(0.5, 0.5, 0.6, 1.0))),
                ));
            });

            // Right Column: Action Button
            p.spawn(Button)
                .insert(Node {
                    width: Val::Px(140.0),
                    height: Val::Px(42.0),
                    border: UiRect::all(Val::Px(1.5)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                })
                .insert(BorderColor::all(Color::Srgba(Srgba::new(
                    0.2, 0.5, 0.3, 0.6,
                ))))
                .insert(BackgroundColor(Color::Srgba(Srgba::new(
                    0.1, 0.22, 0.15, 0.9,
                ))))
                .insert(NextStepButton)
                .with_children(|btn: &mut ChildSpawnerCommands| {
                    btn.spawn((
                        Text::new("NEXT STEP"),
                        TextFont {
                            font_size: 14.0,
                            font: font.clone(),
                            ..default()
                        },
                        TextColor(Color::Srgba(Srgba::WHITE)),
                    ));
                });
        });
}
