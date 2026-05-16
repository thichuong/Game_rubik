pub mod resources;
pub mod systems;

use bevy::prelude::*;
use resources::DragState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>().add_systems(
            Update,
            (systems::handle_mouse_input, systems::handle_keyboard_input),
        );
    }
}
