pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use resources::{FaceMapping, MoveHistory, RotationQueue, RubikSize, RubikSkin};

pub struct RubikPlugin;

impl Plugin for RubikPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RotationQueue>()
            .init_resource::<MoveHistory>()
            .init_resource::<RubikSkin>()
            .init_resource::<RubikSize>()
            .init_resource::<FaceMapping>()
            .add_systems(
                Startup,
                (systems::setup_materials, systems::spawn_rubik_cube).chain(),
            )
            .add_systems(
                Update,
                (
                    systems::handle_rubik_resize,
                    systems::handle_rotation_queue,
                    systems::animate_rotation,
                    systems::update_rubik_rotation,
                    systems::handle_cube_reset,
                    systems::update_skins,
                    systems::update_face_labels,
                ),
            );
    }
}
