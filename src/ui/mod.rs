pub mod systems;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup_ui).add_systems(
            Update,
            (
                systems::handle_shuffle_button,
                systems::handle_solve_button,
                systems::handle_next_step_button,
                systems::handle_close_button,
                systems::update_solution_panel,
                systems::handle_skin_button,
                systems::handle_skin_toggle,
            ),
        );
    }
}
