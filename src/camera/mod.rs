pub mod components;
pub mod systems;

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (systems::update_camera_orbit, systems::handle_camera_reset),
        );
    }
}
