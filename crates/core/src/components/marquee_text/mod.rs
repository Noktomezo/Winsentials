use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, Styled, TextRun, Window, div,
    ease_in_out, linear_color_stop, linear_gradient, px,
};

pub const DEFAULT_MARQUEE_DURATION: Duration = Duration::from_millis(2_400);
pub const DEFAULT_FADE_WIDTH: Pixels = px(8.0);
pub const DEFAULT_FONT_SIZE: Pixels = px(13.0);

/// Smooth back-and-forth easing with gentle pauses at both boundaries.
#[must_use]
pub fn marquee_ping_pong_easing(t: f32) -> f32 {
    const PAUSE: f32 = 0.15;
    const MOVE: f32 = 0.5 - PAUSE;

    let t = t.clamp(0.0, 1.0);

    if t <= PAUSE || t >= 1.0 - 1e-6 {
        0.0
    } else if t < 0.5 {
        let local_t = (t - PAUSE) / MOVE;
        ease_in_out(local_t)
    } else if t <= 0.5 + PAUSE {
        1.0
    } else {
        let local_t = ((t - (0.5 + PAUSE)) / MOVE).clamp(0.0, 1.0);
        ease_in_out(1.0 - local_t)
    }
}

/// Measures text pixel width using GPUI's text shaping system.
#[must_use]
pub fn measure_text_width(
    text: &SharedString,
    font_size: Pixels,
    font_weight: FontWeight,
    window: &mut Window,
) -> Pixels {
    let mut font = window.text_style().font();
    font.weight = font_weight;
    let text_run = TextRun {
        len: text.len(),
        font,
        color: window.text_style().color,
        ..Default::default()
    };
    let line = window
        .text_system()
        .shape_line(text.clone(), font_size, &[text_run], None);
    line.width
}

#[must_use]
pub fn marquee_shift(text_width: Pixels, viewport_width: Pixels) -> Pixels {
    if text_width > viewport_width {
        text_width - viewport_width
    } else {
        Pixels::ZERO
    }
}

/// Animation sync mode for the marquee overflow fog layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MarqueeFadeMode {
    #[default]
    Steady,
    Opening(Duration),
    Closing(Duration),
}

/// Production-grade `MarqueeText` component with smooth back-and-forth motion and edge fog.
///
/// Features:
/// - Smooth back-and-forth ping-pong animation on hover with subtle pauses at boundaries
/// - Soft gradient edge fog extending into gutters strictly preserving surrounding icons
/// - Zero perpetual redraws: animation is only attached when `active` and text overflows
/// - Full reduced-motion and zero-overhead resting state
#[derive(IntoElement)]
pub struct MarqueeText {
    id: ElementId,
    text: SharedString,
    max_width: Pixels,
    font_size: Pixels,
    font_weight: FontWeight,
    text_color: Option<Rgba>,
    fade_color: Rgba,
    fade_width: Pixels,
    active: bool,
    fade_enabled: bool,
    fade_mode: MarqueeFadeMode,
    duration: Duration,
    debug_name: Option<SharedString>,
}

impl MarqueeText {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>, max_width: Pixels) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            max_width,
            font_size: DEFAULT_FONT_SIZE,
            font_weight: FontWeight::MEDIUM,
            text_color: None,
            fade_color: gpui::rgb(0x001c_1b1a),
            fade_width: DEFAULT_FADE_WIDTH,
            active: false,
            fade_enabled: true,
            fade_mode: MarqueeFadeMode::Steady,
            duration: DEFAULT_MARQUEE_DURATION,
            debug_name: None,
        }
    }

    #[must_use]
    pub fn debug_name(mut self, name: impl Into<SharedString>) -> Self {
        self.debug_name = Some(name.into());
        self
    }

    #[must_use]
    pub const fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    #[must_use]
    pub const fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    #[must_use]
    pub const fn text_color(mut self, color: Rgba) -> Self {
        self.text_color = Some(color);
        self
    }

    #[must_use]
    pub const fn fade_color(mut self, color: Rgba) -> Self {
        self.fade_color = color;
        self
    }

    #[must_use]
    pub const fn fade_width(mut self, width: Pixels) -> Self {
        self.fade_width = width;
        self
    }

    #[must_use]
    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    #[must_use]
    pub const fn fade_enabled(mut self, enabled: bool) -> Self {
        self.fade_enabled = enabled;
        self
    }

    #[must_use]
    pub const fn fade_mode(mut self, mode: MarqueeFadeMode) -> Self {
        self.fade_mode = mode;
        self
    }

    #[must_use]
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

impl RenderOnce for MarqueeText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let text_width = measure_text_width(&self.text, self.font_size, self.font_weight, window);
        let viewport_width = text_width.min(self.max_width);
        let shift = marquee_shift(text_width, viewport_width);
        let fade_width = self.fade_width;
        let fade_color = self.fade_color;
        let dbg = self.debug_name;
        let is_motion_reduced = cx.reduce_motion();

        let mut anchor = div()
            .relative()
            .w(viewport_width)
            .h_full()
            .flex()
            .items_center()
            .flex_none();

        if let Some(ref name) = dbg {
            let sel = format!("{name}_anchor");
            anchor = anchor.debug_selector(move || sel.clone());
        }

        if shift > Pixels::ZERO {
            let mut fade_layer = div().absolute().inset_0();
            if self.active {
                let left_dbg = dbg.as_ref().map(|n| format!("{n}_fade_left"));
                fade_layer =
                    fade_layer.child(edge_fade(FadeEdge::Left, fade_width, fade_color, left_dbg));
            }
            let right_dbg = dbg.as_ref().map(|n| format!("{n}_fade_right"));
            let fade_layer = fade_layer.child(edge_fade(
                FadeEdge::Right,
                fade_width,
                fade_color,
                right_dbg,
            ));

            let mut viewport = expanded_viewport(viewport_width, fade_width);
            if let Some(ref name) = dbg {
                let sel = format!("{name}_viewport");
                viewport = viewport.debug_selector(move || sel.clone());
            }

            let line_dbg = dbg.as_ref().map(|n| format!("{n}_line"));

            if self.active && !is_motion_reduced {
                let text = self.text;
                let font_size = self.font_size;
                let font_weight = self.font_weight;
                let text_color = self.text_color;

                viewport = viewport.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .with_animation(
                            self.id.clone(),
                            Animation::new(self.duration)
                                .repeat()
                                .with_easing(marquee_ping_pong_easing),
                            move |element, progress| {
                                element.child(marquee_line(
                                    text.clone(),
                                    fade_width - shift * progress,
                                    font_size,
                                    font_weight,
                                    text_color,
                                    line_dbg.clone(),
                                ))
                            },
                        ),
                );
            } else {
                viewport = viewport.child(marquee_line(
                    self.text,
                    fade_width,
                    self.font_size,
                    self.font_weight,
                    self.text_color,
                    line_dbg,
                ));
            }

            if self.fade_enabled {
                let animated_fade_layer = match self.fade_mode {
                    MarqueeFadeMode::Steady => fade_layer.into_any_element(),
                    MarqueeFadeMode::Opening(duration) => {
                        let fade_anim_id = format!("{}_fog_open", self.id);
                        fade_layer
                            .with_animation(
                                ElementId::Name(fade_anim_id.into()),
                                Animation::new(duration).with_easing(ease_in_out),
                                gpui::Styled::opacity,
                            )
                            .into_any_element()
                    }
                    MarqueeFadeMode::Closing(duration) => {
                        let fade_anim_id = format!("{}_fog_close", self.id);
                        fade_layer
                            .with_animation(
                                ElementId::Name(fade_anim_id.into()),
                                Animation::new(duration).with_easing(ease_in_out),
                                move |layer, delta| layer.opacity(1.0 - delta),
                            )
                            .into_any_element()
                    }
                };
                viewport = viewport.child(animated_fade_layer);
            }

            anchor.child(viewport)
        } else {
            let line_dbg = dbg.as_ref().map(|n| format!("{n}_line"));
            anchor.child(marquee_line(
                self.text,
                Pixels::ZERO,
                self.font_size,
                self.font_weight,
                self.text_color,
                line_dbg,
            ))
        }
    }
}

/// Viewport expanded by `fade_width` on both sides so fog dissolves overflowing text in surrounding gutters.
fn expanded_viewport(viewport_width: Pixels, fade_width: Pixels) -> Div {
    div()
        .absolute()
        .left(-fade_width)
        .top_0()
        .bottom_0()
        .w(viewport_width + fade_width * 2.0)
        .flex()
        .items_center()
        .overflow_hidden()
}

fn marquee_line(
    text: SharedString,
    offset: Pixels,
    font_size: Pixels,
    font_weight: FontWeight,
    text_color: Option<Rgba>,
    debug_name: Option<String>,
) -> Div {
    let mut line = div()
        .relative()
        .left(offset)
        .flex_none()
        .whitespace_nowrap()
        .text_size(font_size)
        .font_weight(font_weight)
        .child(text);

    if let Some(color) = text_color {
        line = line.text_color(color);
    }
    if let Some(name) = debug_name {
        line = line.debug_selector(move || name.clone());
    }
    line
}

#[derive(Clone, Copy)]
enum FadeEdge {
    Left,
    Right,
}

fn edge_fade(edge: FadeEdge, width: Pixels, color: Rgba, debug_sel: Option<String>) -> Div {
    let transparent = color.opacity(0.0);
    let background = match edge {
        FadeEdge::Left => linear_gradient(
            90.0,
            linear_color_stop(color, 0.0),
            linear_color_stop(transparent, 1.0),
        ),
        FadeEdge::Right => linear_gradient(
            90.0,
            linear_color_stop(transparent, 0.0),
            linear_color_stop(color, 1.0),
        ),
    };

    let mut overlay = div().absolute().top_0().bottom_0().w(width).bg(background);
    if let Some(sel) = debug_sel {
        overlay = overlay.debug_selector(move || sel.clone());
    }

    match edge {
        FadeEdge::Left => overlay.left_0(),
        FadeEdge::Right => overlay.right_0(),
    }
}

#[cfg(test)]
mod tests;
