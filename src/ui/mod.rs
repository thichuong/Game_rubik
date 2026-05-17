pub mod components;
pub mod interactions;
pub mod layout;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, layout::setup_ui).add_systems(
            Update,
            (
                interactions::handle_shuffle_button,
                interactions::handle_solve_button,
                interactions::handle_next_step_button,
                interactions::handle_close_button,
                interactions::update_solution_panel,
                interactions::handle_skin_button,
                interactions::handle_skin_toggle,
                interactions::handle_env_toggle,
                interactions::handle_env_controls,
                interactions::handle_size_slider_track,
                interactions::handle_size_decrement_button,
                interactions::handle_size_increment_button,
                interactions::update_size_slider_ui,
                interactions::update_solve_button_state,
                interactions::debug_all_interactions,
            ),
        );
    }
}
