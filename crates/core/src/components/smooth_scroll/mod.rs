use std::time::Instant;

use gpui::{
    AnyElement, App, Bounds, Entity, Global, IntoElement, Pixels, RenderOnce, ScrollHandle, Window,
    point, px,
};

#[cfg(test)]
mod tests;
mod viewport;
mod virtual_list;

pub(crate) use viewport::{render_scroll_viewport, schedule_animation};
pub use virtual_list::{SmoothVirtualList, VirtualItemRenderer};

const SCROLL_DAMPING: f32 = 6.0;
const WIDTH_DAMPING: f32 = 18.0;
const TRACK_PADDING: Pixels = px(4.0);
const MIN_THUMB_HEIGHT: Pixels = px(32.0);

#[derive(Default)]
pub struct SmoothScrollRegistry {
    states: std::collections::HashMap<&'static str, Entity<SmoothScrollState>>,
}

impl Global for SmoothScrollRegistry {}

impl SmoothScrollRegistry {
    pub fn register(id: &'static str, state: Entity<SmoothScrollState>, cx: &mut App) {
        if cx.has_global::<Self>() {
            cx.global_mut::<Self>().states.insert(id, state);
        } else {
            let mut states = std::collections::HashMap::new();
            states.insert(id, state);
            cx.set_global(Self { states });
        }
    }

    #[must_use]
    pub fn get(id: &'static str, cx: &App) -> Option<Entity<SmoothScrollState>> {
        cx.try_global::<Self>()
            .and_then(|reg| reg.states.get(id).cloned())
    }
}

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
        let Some(state) = SmoothScrollRegistry::get(id, cx) else {
            return false;
        };
        let (viewport_bounds, current_offset, max_offset) = {
            let s = state.read(cx);
            (
                s.handle.bounds(),
                s.handle.offset().y,
                s.handle.max_offset().y,
            )
        };

        let viewport_top = if viewport_bounds.size.height > px(0.0) {
            viewport_bounds.top()
        } else {
            px(44.0)
        };
        let viewport_bottom = if viewport_bounds.size.height > px(0.0) {
            viewport_bounds.bottom()
        } else {
            window.viewport_size().height
        };

        let item_top = item_bounds.top();
        let item_bottom = item_bounds.bottom();

        if item_top >= viewport_top + margin && item_bottom <= viewport_bottom - margin {
            return true;
        }

        let content_y = item_top - viewport_top - current_offset;
        let scroll_distance = (content_y - margin).max(px(0.0));
        let target_y = if max_offset > px(0.0) {
            -scroll_distance.min(max_offset)
        } else {
            -scroll_distance
        };

        let reduce_motion = cx.reduce_motion();
        let should_animate = state.update(cx, |s, cx| {
            let anim = s.scroll_to(target_y, reduce_motion);
            if reduce_motion {
                cx.notify();
            }
            anim
        });
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
        let target = if max_offset > px(0.0) {
            target_y.clamp(-max_offset, px(0.0))
        } else {
            target_y.min(px(0.0))
        };
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

impl RenderOnce for SmoothScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_scroll_viewport(self.id, self.child, window, cx)
    }
}
