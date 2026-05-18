pub mod components;
pub mod interactions;
pub mod layout;

use crate::ui::components::SidebarScrollState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidebarScrollState>()
            .add_systems(Startup, layout::setup_ui)
            .add_systems(
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
                ),
            )
            .add_systems(
                Update,
                (
                    interactions::handle_env_controls,
                    interactions::handle_size_slider_track,
                    interactions::handle_size_decrement_button,
                    interactions::handle_size_increment_button,
                    interactions::update_size_slider_ui,
                    interactions::update_solve_button_state,
                    interactions::handle_exit_button,
                ),
            )
            .add_systems(
                Update,
                (
                    interactions::handle_mapping_toggle,
                    interactions::handle_mapping_controls,
                    interactions::update_mapping_ui,
                    interactions::handle_sidebar_scroll,
                    interactions::update_sidebar_scrollbar_visuals,
                    interactions::handle_sidebar_scrollbar_drag,
                    interactions::handle_camera_toggle,
                    interactions::update_camera_feed,
                ),
            );
    }
}
