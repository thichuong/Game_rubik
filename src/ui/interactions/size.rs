use crate::rubik::resources::RubikSize;
use crate::ui::components::{
    SizeDecrementButton, SizeIncrementButton, SizeSliderFill, SizeSliderHandle, SizeSliderTrack,
    SizeText,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rubik_solver::StepByStepSolution;

/// System to handle changes in the Rubik size via the Slider Track
#[allow(clippy::cast_possible_truncation)]
pub fn handle_size_slider_track(
    mut rubik_size: ResMut<RubikSize>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    track_query: Query<
        (&Interaction, &bevy::ui::UiGlobalTransform, &ComputedNode),
        With<SizeSliderTrack>,
    >,
    mut is_dragging: Local<bool>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut solution: ResMut<StepByStepSolution>,
) {
    // Disable size adjustment if solver is active or searching
    if solution.is_searching || (solution.active && solution.current_step < solution.moves.len()) {
        *is_dragging = false;
        return;
    }

    let Some((track_interaction, transform, computed_node)) = track_query.iter().next() else {
        return;
    };

    if matches!(*track_interaction, Interaction::Pressed) {
        *is_dragging = true;
    }

    if mouse_input.just_released(MouseButton::Left) {
        *is_dragging = false;
    }

    if *is_dragging {
        let window = *window_query;
        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };

        let width = computed_node.size().x;
        let center_x = transform.translation.x;
        let left_x = center_x - width / 2.0;

        let pct = ((cursor_pos.x - left_x) / width).clamp(0.0, 1.0);
        let new_size = 2 + (pct * 10.0).round() as i32;

        if rubik_size.size != new_size {
            rubik_size.size = new_size;
            if solution.active {
                solution.active = false;
            }
        }
    }
}

/// System to handle fast decrement button (-) for cube size
pub fn handle_size_decrement_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SizeDecrementButton>)>,
    mut rubik_size: ResMut<RubikSize>,
    mut solution: ResMut<StepByStepSolution>,
) {
    // Block sizing while solving
    if solution.is_searching || (solution.active && solution.current_step < solution.moves.len()) {
        return;
    }

    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) && rubik_size.size > 2 {
            rubik_size.size -= 1;
            if solution.active {
                solution.active = false;
            }
        }
    }
}

/// System to handle fast increment button (+) for cube size
pub fn handle_size_increment_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SizeIncrementButton>)>,
    mut rubik_size: ResMut<RubikSize>,
    mut solution: ResMut<StepByStepSolution>,
) {
    // Block sizing while solving
    if solution.is_searching || (solution.active && solution.current_step < solution.moves.len()) {
        return;
    }

    for interaction in &mut interaction_query {
        if matches!(*interaction, Interaction::Pressed) && rubik_size.size < 12 {
            rubik_size.size += 1;
            if solution.active {
                solution.active = false;
            }
        }
    }
}

/// System to update the visual representation of the Slider (fill width & handle position)
#[allow(clippy::cast_precision_loss)]
pub fn update_size_slider_ui(
    rubik_size: Res<RubikSize>,
    mut size_text_query: Single<&mut Text, With<SizeText>>,
    mut fill_query: Single<&mut Node, (With<SizeSliderFill>, Without<SizeSliderHandle>)>,
    mut handle_query: Single<&mut Node, (With<SizeSliderHandle>, Without<SizeSliderFill>)>,
) {
    if rubik_size.is_changed() {
        let size = rubik_size.size;
        size_text_query.0 = format!("{size}x{size}x{size}");

        // Map size 2..=12 to 0%..=100% progress
        let pct = (size - 2) as f32 / 10.0 * 100.0;
        fill_query.width = Val::Percent(pct);
        handle_query.left = Val::Percent(pct);
    }
}
