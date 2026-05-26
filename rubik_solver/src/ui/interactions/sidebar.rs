use crate::ui::components::{
    ScrollContentWrapper, SidebarScrollHandle, SidebarScrollState, SidebarScrollable,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn handle_sidebar_scroll(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<
        (
            &bevy::ui::UiGlobalTransform,
            &ComputedNode,
            &mut ScrollPosition,
        ),
        With<SidebarScrollable>,
    >,
    content_query: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    viewport_query: Option<Single<&ComputedNode, With<SidebarScrollable>>>,
    window_query: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    let Some(window) = window_query else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let mut scroll_dy = 0.0;
    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 35.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
        };
        scroll_dy -= dy;
    }

    if scroll_dy != 0.0 {
        let content_height = if let Some(node) = content_query {
            node.size.y
        } else {
            0.0
        };
        let viewport_height = if let Some(node) = viewport_query {
            node.size.y
        } else {
            0.0
        };
        let max_scroll = (content_height - viewport_height).max(0.0);

        for (transform, computed_node, mut scroll_pos) in &mut query {
            let size = computed_node.size();
            let center_x = transform.translation.x;
            let center_y = transform.translation.y;
            let half_w = size.x / 2.0;
            let half_h = size.y / 2.0;

            let is_hovered = cursor_pos.x >= center_x - half_w
                && cursor_pos.x <= center_x + half_w
                && cursor_pos.y >= center_y - half_h
                && cursor_pos.y <= center_y + half_h;

            if is_hovered {
                scroll_pos.0.y = (scroll_pos.0.y + scroll_dy).clamp(0.0, max_scroll);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_sidebar_scrollbar_visuals(
    scroll_data: Option<Single<(&ComputedNode, &ScrollPosition), With<SidebarScrollable>>>,
    content_node: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    handle_data: Option<
        Single<(&Interaction, &mut Node, &mut BackgroundColor), With<SidebarScrollHandle>>,
    >,
    scroll_state: Res<SidebarScrollState>,
) {
    let Some(scroll_data) = scroll_data else {
        return;
    };
    let Some(content_node) = content_node else {
        return;
    };
    let Some(mut handle_data) = handle_data else {
        return;
    };

    let viewport_height = scroll_data.0.size.y;
    let scroll_pos = scroll_data.1;
    let content_height = content_node.size.y;

    if content_height <= viewport_height {
        handle_data.1.display = Display::None;
        return;
    }

    handle_data.1.display = Display::Flex;

    let track_height = viewport_height - 20.0; // 10px top/bottom padding
    let handle_height = ((viewport_height / content_height) * track_height).max(30.0);

    let max_scroll = content_height - viewport_height;
    let ratio = if max_scroll > 0.0 {
        scroll_pos.0.y / max_scroll
    } else {
        0.0
    };
    let handle_top = ratio * (track_height - handle_height);

    handle_data.1.height = Val::Px(handle_height);
    handle_data.1.top = Val::Px(handle_top);

    let interaction = handle_data.0;

    // Dynamic background color based on interaction state
    if scroll_state.is_dragging {
        *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.6, 0.9, 0.9)));
    } else {
        match *interaction {
            Interaction::Pressed => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.4, 0.6, 0.9, 0.9)));
            }
            Interaction::Hovered => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.35, 0.35, 0.45, 0.8)));
            }
            Interaction::None => {
                *handle_data.2 = BackgroundColor(Color::Srgba(Srgba::new(0.25, 0.25, 0.35, 0.55)));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_sidebar_scrollbar_drag(
    mut scroll_state: ResMut<SidebarScrollState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Option<Single<&Window, With<PrimaryWindow>>>,
    scroll_data: Option<Single<(&ComputedNode, &mut ScrollPosition), With<SidebarScrollable>>>,
    content_node: Option<Single<&ComputedNode, With<ScrollContentWrapper>>>,
    handle_data: Option<Single<(&Interaction, &ComputedNode), With<SidebarScrollHandle>>>,
) {
    let Some(mut scroll_data) = scroll_data else {
        return;
    };
    let Some(content_node) = content_node else {
        return;
    };
    let Some(handle_data) = handle_data else {
        return;
    };

    let Some(window) = windows else {
        return;
    };
    let cursor_y = if let Some(pos) = window.cursor_position() {
        pos.y
    } else {
        return;
    };

    let viewport_height = scroll_data.0.size.y;
    let content_height = content_node.size.y;
    let handle_height = handle_data.1.size.y;

    let max_scroll = (content_height - viewport_height).max(0.0);
    if max_scroll <= 0.0 {
        scroll_state.is_dragging = false;
        return;
    }

    let track_height = viewport_height - 20.0;
    let scrollable_track_range = (track_height - handle_height).max(1.0);

    let interaction = handle_data.0;

    if mouse_input.just_pressed(MouseButton::Left) && *interaction == Interaction::Pressed {
        scroll_state.is_dragging = true;
        scroll_state.drag_start_cursor_y = cursor_y;
        scroll_state.drag_start_scroll_y = scroll_data.1.0.y;
    }

    if scroll_state.is_dragging {
        if mouse_input.pressed(MouseButton::Left) {
            let delta_cursor_y = cursor_y - scroll_state.drag_start_cursor_y;
            let delta_scroll_y = delta_cursor_y * (max_scroll / scrollable_track_range);
            scroll_data.1.0.y =
                (scroll_state.drag_start_scroll_y + delta_scroll_y).clamp(0.0, max_scroll);
        } else {
            scroll_state.is_dragging = false;
        }
    }
}
