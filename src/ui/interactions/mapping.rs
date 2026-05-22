use crate::rubik::components::Face;
use crate::rubik::resources::FaceMapping;
use crate::ui::components::{MappingControl, MappingList, MappingOrderText, MappingToggleButton};
use bevy::prelude::*;

pub type MappingToggleQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<MappingToggleButton>),
>;

pub fn handle_mapping_toggle(
    mut interaction_query: MappingToggleQuery,
    mut mapping_list: Single<&mut Node, With<MappingList>>,
    mut state: Local<bool>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *state = !*state;
                mapping_list.display = if *state { Display::Flex } else { Display::None };
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

pub type MappingControlQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static MappingControl,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<Button>),
>;

pub fn handle_mapping_controls(
    mut interaction_query: MappingControlQuery,
    mut mapping: ResMut<FaceMapping>,
) {
    for (interaction, control, mut bg_color) in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) {
            match *control {
                MappingControl::ToggleOrder => {
                    mapping.select_d_first = !mapping.select_d_first;
                }
                MappingControl::SelectF(face) => {
                    if mapping.select_d_first {
                        // D First: only allow if perpendicular to current D
                        if mapping.d_face.normal().dot(face.normal()).abs() < 0.1 {
                            mapping.f_face = face;
                        }
                    } else {
                        // F First: allow any selection, auto-resolve D if conflict
                        mapping.f_face = face;
                        if mapping.d_face.normal().dot(face.normal()).abs() > 0.9 {
                            // Conflict! Auto-select a perpendicular face
                            mapping.d_face = if face == Face::Up || face == Face::Down {
                                Face::Front
                            } else {
                                Face::Up
                            };
                        }
                    }
                }
                MappingControl::SelectD(face) => {
                    if mapping.select_d_first {
                        // D First: allow any selection, auto-resolve F if conflict
                        mapping.d_face = face;
                        if mapping.f_face.normal().dot(face.normal()).abs() > 0.9 {
                            // Conflict! Auto-select a perpendicular face
                            mapping.f_face = if face == Face::Up || face == Face::Down {
                                Face::Front
                            } else {
                                Face::Up
                            };
                        }
                    } else {
                        // F First: only allow if perpendicular to current F
                        if mapping.f_face.normal().dot(face.normal()).abs() < 0.1 {
                            mapping.d_face = face;
                        }
                    }
                }
            }
            *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.4, 0.5, 1.0)));
        }
    }
}

pub fn update_mapping_ui(
    mapping: Res<FaceMapping>,
    mut order_text_query: Query<&mut Text, With<MappingOrderText>>,
    mut button_query: Query<
        (
            &MappingControl,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    if mapping.is_changed() {
        // 1. Update priority text
        for mut text in &mut order_text_query {
            text.0 = if mapping.select_d_first {
                "Priority: D First".to_string()
            } else {
                "Priority: F First".to_string()
            };
        }

        // 2. Update button styles based on selected FaceMapping
        for (control, mut node, mut bg_color, mut border_color) in &mut button_query {
            match *control {
                MappingControl::ToggleOrder => {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.12, 0.15, 0.25, 0.85)));
                    *border_color =
                        BorderColor::all(Color::Srgba(Srgba::new(0.25, 0.35, 0.55, 0.6)));
                    node.display = Display::Flex;
                }
                MappingControl::SelectF(face) => {
                    let is_selected = mapping.f_face == face;

                    if mapping.select_d_first {
                        // D First: F is the second choice (only show 4 perpendicular to D)
                        let is_disabled = mapping.d_face.normal().dot(face.normal()).abs() > 0.9;
                        if is_disabled {
                            node.display = Display::None;
                        } else {
                            node.display = Display::Flex;
                            if is_selected {
                                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(
                                    0.15, 0.45, 0.25, 0.85,
                                )));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.8, 0.4, 0.9)));
                            } else {
                                *bg_color =
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                            }
                        }
                    } else {
                        // F First: F is the first choice (show all 6)
                        node.display = Display::Flex;
                        if is_selected {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.45, 0.25, 0.85)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.8, 0.4, 0.9)));
                        } else {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                        }
                    }
                }
                MappingControl::SelectD(face) => {
                    let is_selected = mapping.d_face == face;

                    if mapping.select_d_first {
                        // D First: D is the first choice (show all 6)
                        node.display = Display::Flex;
                        if is_selected {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.45, 0.35, 0.1, 0.85)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.8, 0.6, 0.2, 0.9)));
                        } else {
                            *bg_color =
                                BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                            *border_color =
                                BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                        }
                    } else {
                        // F First: D is the second choice (only show 4 perpendicular to F)
                        let is_disabled = mapping.f_face.normal().dot(face.normal()).abs() > 0.9;
                        if is_disabled {
                            node.display = Display::None;
                        } else {
                            node.display = Display::Flex;
                            if is_selected {
                                *bg_color = BackgroundColor(Color::Srgba(Srgba::new(
                                    0.45, 0.35, 0.1, 0.85,
                                )));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.8, 0.6, 0.2, 0.9)));
                            } else {
                                *bg_color =
                                    BackgroundColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 0.7)));
                                *border_color =
                                    BorderColor::all(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.4)));
                            }
                        }
                    }
                }
            }
        }
    }
}
