use std::time::Instant;

use gpui::{
    AnyElement, App, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use super::{
    SmoothScrollRegistry, SmoothScrollState, TRACK_PADDING, offset_from_thumb, thumb_geometry,
};
use crate::theme::Theme;

pub(crate) fn schedule_animation(state: Entity<SmoothScrollState>, window: &Window) {
    window.on_next_frame(move |window, cx| {
        let keep_animating = state.update(cx, |state, cx| {
            let keep_animating = state.advance(Instant::now());
            cx.notify();
            keep_animating
        });
        if keep_animating {
            schedule_animation(state, window);
        }
    });
}

#[allow(clippy::too_many_lines)]
pub(crate) fn render_scroll_viewport(
    id: &'static str,
    content: AnyElement,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = Theme::get(cx);
    let state = window.use_keyed_state((id, 0usize), cx, |_, _| SmoothScrollState::default());
    SmoothScrollRegistry::register(id, state.clone(), cx);
    let handle = state.read(cx).handle.clone();
    let geometry = thumb_geometry(
        handle.bounds().size.height,
        handle.max_offset().y,
        handle.offset().y,
    );

    let wheel_state = state.clone();
    let viewport = div()
        .id((id, 1usize))
        .size_full()
        .overflow_y_hidden()
        .track_scroll(&handle)
        .on_scroll_wheel(move |event, window, cx| {
            if event.modifiers.control {
                return;
            }
            let delta = event.delta.pixel_delta(window.line_height()).y;
            let (handled, start) = wheel_state.update(cx, |state, cx| {
                let result = state.scroll_by(delta, cx.reduce_motion());
                if result.0 {
                    cx.notify();
                }
                result
            });
            if handled {
                cx.stop_propagation();
            }
            if start {
                schedule_animation(wheel_state.clone(), window);
            }
        })
        .child(content);

    let mut root = div()
        .relative()
        .size_full()
        .overflow_hidden()
        .child(viewport);

    if state.read(cx).dragging.is_some() {
        let move_state = state.clone();
        let up_state = state.clone();
        root = root.child(
            div()
                .id((id, 2usize))
                .absolute()
                .inset_0()
                .on_mouse_move(move |event, _window, cx| {
                    move_state.update(cx, |state, cx| {
                        let Some(grab_offset) = state.dragging else {
                            return;
                        };
                        let viewport = state.handle.bounds();
                        let Some((_, thumb_height)) = thumb_geometry(
                            viewport.size.height,
                            state.handle.max_offset().y,
                            state.handle.offset().y,
                        ) else {
                            return;
                        };
                        let track_height = viewport.size.height - TRACK_PADDING * 2.0;
                        let pointer_y = event.position.y - viewport.origin.y - TRACK_PADDING;
                        let offset = offset_from_thumb(
                            pointer_y,
                            grab_offset,
                            track_height,
                            thumb_height,
                            state.handle.max_offset().y,
                        );
                        state.set_offset(offset);
                        cx.notify();
                    });
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Left, move |_, _window, cx| {
                    up_state.update(cx, |state, cx| {
                        state.dragging = None;
                        cx.notify();
                    });
                    cx.stop_propagation();
                }),
        );
    }

    if let Some((thumb_top, thumb_height)) = geometry {
        let hover_state = state.clone();
        let track_state = state.clone();
        let thumb_state = state.clone();
        let thumb_width = state.read(cx).thumb_width;
        let thumb_color = if state.read(cx).hovered || state.read(cx).dragging.is_some() {
            theme.text_muted.opacity(0.65)
        } else {
            theme.text_muted.opacity(0.38)
        };

        root = root.child(
            div()
                .id((id, 3usize))
                .absolute()
                .top_0()
                .right(px(2.0))
                .h_full()
                .w(px(14.0))
                .on_hover(move |hovered, window, cx| {
                    let start = hover_state.update(cx, |state, cx| {
                        let changed = state.set_hovered(*hovered, cx.reduce_motion());
                        if changed {
                            cx.notify();
                        }
                        changed && state.animating
                    });
                    if start {
                        schedule_animation(hover_state.clone(), window);
                    }
                })
                .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                    track_state.update(cx, |state, cx| {
                        let viewport = state.handle.bounds();
                        let pointer_y = event.position.y - viewport.origin.y - TRACK_PADDING;
                        let track_height = viewport.size.height - TRACK_PADDING * 2.0;
                        let offset = offset_from_thumb(
                            pointer_y,
                            thumb_height * 0.5,
                            track_height,
                            thumb_height,
                            state.handle.max_offset().y,
                        );
                        state.set_offset(offset);
                        cx.notify();
                    });
                    cx.stop_propagation();
                })
                .child(
                    div()
                        .id((id, 4usize))
                        .absolute()
                        .top(TRACK_PADDING + thumb_top)
                        .right((px(12.0) - thumb_width) / 2.0)
                        .h(thumb_height)
                        .w(thumb_width)
                        .rounded(px(4.0))
                        .bg(thumb_color)
                        .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                            thumb_state.update(cx, |state, cx| {
                                let viewport = state.handle.bounds();
                                let Some((thumb_top, _)) = thumb_geometry(
                                    viewport.size.height,
                                    state.handle.max_offset().y,
                                    state.handle.offset().y,
                                ) else {
                                    return;
                                };
                                state.dragging = Some(
                                    event.position.y
                                        - viewport.origin.y
                                        - TRACK_PADDING
                                        - thumb_top,
                                );
                                state.target_y = state.handle.offset().y;
                                cx.notify();
                            });
                            cx.stop_propagation();
                        }),
                ),
        );
    }

    root
}
