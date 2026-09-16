use std::time::Instant;

use gpui::{
    AnyElement, App, Bounds, Entity, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, RenderOnce, ScrollHandle, StatefulInteractiveElement, Styled, Window, div, point, px,
};

use crate::theme::Theme;

#[cfg(test)]
mod tests;
mod virtual_list;

pub use virtual_list::{SmoothVirtualList, VirtualItemRenderer};

const SCROLL_DAMPING: f32 = 6.0;
const WIDTH_DAMPING: f32 = 18.0;
const TRACK_PADDING: Pixels = px(4.0);
const MIN_THUMB_HEIGHT: Pixels = px(32.0);

#[derive(IntoElement)]
pub struct SmoothScroll {
    id: &'static str,
    child: AnyElement,
}

impl SmoothScroll {
    #[must_use]
    pub fn new(id: &'static str, child: impl IntoElement) -> Self {
        Self {
            id,
            child: child.into_any_element(),
        }
    }

    pub fn scroll_bounds_into_view(
        id: &'static str,
        item_bounds: Bounds<Pixels>,
        margin: Pixels,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let state = window.use_keyed_state((id, 0usize), cx, |_, _| SmoothScrollState::default());
        let (viewport_bounds, current_offset, max_offset) = {
            let s = state.read(cx);
            (
                s.handle.bounds(),
                s.handle.offset().y,
                s.handle.max_offset().y,
            )
        };

        if viewport_bounds.size.height <= px(0.0) {
            return false;
        }

        let viewport_top = viewport_bounds.top();
        let viewport_bottom = viewport_bounds.bottom();
        let item_top = item_bounds.top();
        let item_bottom = item_bounds.bottom();

        if item_top >= viewport_top + margin && item_bottom <= viewport_bottom - margin {
            return true;
        }

        if max_offset <= px(0.0) {
            return true;
        }

        let content_y = item_top - viewport_top - current_offset;
        let target_y = -(content_y - margin).clamp(px(0.0), max_offset);

        let reduce_motion = cx.reduce_motion();
        let should_animate = state.update(cx, |s, _cx| s.scroll_to(target_y, reduce_motion));
        if should_animate {
            schedule_animation(state, window);
        }
        true
    }
}

pub struct SmoothScrollState {
    pub handle: ScrollHandle,
    pub target_y: Pixels,
    pub thumb_width: Pixels,
    pub hovered: bool,
    pub dragging: Option<Pixels>,
    pub animating: bool,
    pub last_frame: Instant,
}

impl Default for SmoothScrollState {
    fn default() -> Self {
        Self {
            handle: ScrollHandle::new(),
            target_y: px(0.0),
            thumb_width: px(6.0),
            hovered: false,
            dragging: None,
            animating: false,
            last_frame: Instant::now(),
        }
    }
}

impl SmoothScrollState {
    pub fn begin_animation(&mut self) -> bool {
        if self.animating {
            false
        } else {
            self.animating = true;
            self.last_frame = Instant::now();
            true
        }
    }

    pub fn scroll_to(&mut self, target_y: Pixels, reduce_motion: bool) -> bool {
        let max_offset = self.handle.max_offset().y;
        let target = target_y.clamp(-max_offset, px(0.0));
        if (self.target_y - target).abs() < px(1.0)
            && (self.handle.offset().y - target).abs() < px(1.0)
        {
            return false;
        }
        self.target_y = target;
        if reduce_motion {
            self.handle.set_offset(point(px(0.0), target));
            self.animating = false;
            false
        } else {
            self.begin_animation()
        }
    }

    fn scroll_by(&mut self, delta: Pixels, reduce_motion: bool) -> (bool, bool) {
        let max_offset = self.handle.max_offset().y;
        let current = self.handle.offset().y;
        let base = if self.animating {
            self.target_y
        } else {
            current
        };
        let Some(target) = scroll_target(base, delta, max_offset) else {
            return (false, false);
        };

        self.target_y = target;
        if reduce_motion {
            self.handle.set_offset(point(px(0.0), target));
            self.animating = false;
            (true, false)
        } else {
            (true, self.begin_animation())
        }
    }

    fn set_hovered(&mut self, hovered: bool, reduce_motion: bool) -> bool {
        if self.hovered == hovered {
            return false;
        }
        self.hovered = hovered;
        if reduce_motion {
            self.thumb_width = if hovered { px(8.0) } else { px(6.0) };
            false
        } else {
            self.begin_animation()
        }
    }

    fn set_offset(&mut self, offset: Pixels) {
        let offset = offset.clamp(-self.handle.max_offset().y, px(0.0));
        self.target_y = offset;
        self.handle.set_offset(point(px(0.0), offset));
    }

    fn advance(&mut self, now: Instant) -> bool {
        let delta_time = now
            .duration_since(self.last_frame)
            .as_secs_f32()
            .clamp(0.0, 0.05);
        self.last_frame = now;

        let max_offset = self.handle.max_offset().y;
        self.target_y = self.target_y.clamp(-max_offset, px(0.0));
        let current_y = self.handle.offset().y;
        let next_y =
            current_y + (self.target_y - current_y) * damping_factor(SCROLL_DAMPING, delta_time);
        let scroll_done = (self.target_y - next_y).abs() <= px(0.5);
        self.handle.set_offset(point(
            px(0.0),
            if scroll_done { self.target_y } else { next_y },
        ));

        let target_width = if self.hovered { px(8.0) } else { px(6.0) };
        self.thumb_width +=
            (target_width - self.thumb_width) * damping_factor(WIDTH_DAMPING, delta_time);
        let width_done = (target_width - self.thumb_width).abs() <= px(0.02);
        if width_done {
            self.thumb_width = target_width;
        }

        self.animating = !(scroll_done && width_done);
        self.animating
    }
}

pub(crate) fn scroll_target(base: Pixels, delta: Pixels, max_offset: Pixels) -> Option<Pixels> {
    if max_offset <= px(0.0) || delta == px(0.0) {
        return None;
    }
    let target = (base + delta).clamp(-max_offset, px(0.0));
    (target != base).then_some(target)
}

pub(crate) fn damping_factor(lambda: f32, delta_time: f32) -> f32 {
    1.0 - (-lambda * delta_time).exp()
}

pub(crate) fn thumb_geometry(
    viewport_height: Pixels,
    max_offset: Pixels,
    offset: Pixels,
) -> Option<(Pixels, Pixels)> {
    if viewport_height <= TRACK_PADDING * 2.0 || max_offset <= px(0.0) {
        return None;
    }

    let track_height = viewport_height - TRACK_PADDING * 2.0;
    let content_height = viewport_height + max_offset;
    let thumb_height = (track_height * (viewport_height / content_height))
        .max(MIN_THUMB_HEIGHT)
        .min(track_height);
    let travel = track_height - thumb_height;
    let progress = (-offset / max_offset).clamp(0.0, 1.0);
    Some((travel * progress, thumb_height))
}

pub(crate) fn offset_from_thumb(
    pointer_y: Pixels,
    grab_offset: Pixels,
    track_height: Pixels,
    thumb_height: Pixels,
    max_offset: Pixels,
) -> Pixels {
    let travel = track_height - thumb_height;
    if travel <= px(0.0) {
        px(0.0)
    } else {
        -max_offset * ((pointer_y - grab_offset).clamp(px(0.0), travel) / travel)
    }
}

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

impl RenderOnce for SmoothScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_scroll_viewport(self.id, self.child, window, cx)
    }
}
