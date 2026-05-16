pub mod resources;
pub mod systems;

use bevy::prelude::*;
use resources::EnvironmentSettings;
use systems::{setup_environment, update_environment, update_light_intensity};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnvironmentSettings>()
            .add_systems(Startup, setup_environment)
            .add_systems(Update, (update_environment, update_light_intensity));
    }
}
