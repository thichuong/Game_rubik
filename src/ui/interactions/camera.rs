
use crate::events::CameraFrameEvent;
use crate::input::hand_tracking::HandTrackingEnabled;
use crate::ui::components::{CameraFeedImage, CameraTrackingButton, CameraTrackingText};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub type CameraToggleQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<Interaction>, With<CameraTrackingButton>),
>;

pub fn handle_camera_toggle(
    mut interaction_query: CameraToggleQuery,
    mut text_query: Single<&mut Text, With<CameraTrackingText>>,
    mut image_node: Option<Single<&mut Node, With<CameraFeedImage>>>,
    mut enabled: ResMut<HandTrackingEnabled>,
) {
    for (interaction, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                enabled.0 = !enabled.0;
                if enabled.0 {
                    text_query.0 = "CAMERA: ON".to_string();
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.6, 0.3, 0.85)));
                    *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.4, 0.9, 0.5, 0.9)));
                    if let Some(ref mut img) = image_node {
                        img.display = Display::Flex;
                    }
                } else {
                    text_query.0 = "CAMERA: OFF".to_string();
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.8)));
                    *border_color = BorderColor::all(Color::Srgba(Srgba::new(0.3, 0.3, 0.4, 0.5)));
                    if let Some(ref mut img) = image_node {
                        img.display = Display::None;
                    }
                }
            }
            Interaction::Hovered => {
                if !enabled.0 {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.2, 0.2, 0.25, 0.9)));
                }
            }
            Interaction::None => {
                if !enabled.0 {
                    *bg_color = BackgroundColor(Color::Srgba(Srgba::new(0.15, 0.15, 0.2, 0.8)));
                }
            }
        }
    }
}

pub fn update_camera_feed(
    mut events: MessageReader<CameraFrameEvent>,
    mut images: ResMut<Assets<Image>>,
    mut image_node: Option<Single<&mut ImageNode, With<CameraFeedImage>>>,
) {
    // Only process the latest frame if multiple arrived
    if let Some(latest_event) = events.read().last() {
        if let Some(ref mut img_node) = image_node {
            let image = Image::new(
                Extent3d {
                    width: latest_event.width,
                    height: latest_event.height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                latest_event.frame_rgba.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );

            let handle = images.add(image);
            img_node.image = handle;
        }
    }
}
