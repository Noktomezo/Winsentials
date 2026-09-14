use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, Styled, TextRun, Window, canvas, div,
    ease_in_out, px,
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
/// - Character-level glyph opacity dissolution in gutters to eliminate quad overlay blending
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
                    fade_layer.child(edge_gutter_marker(FadeEdge::Left, fade_width, left_dbg));
            }
            let right_dbg = dbg.as_ref().map(|n| format!("{n}_fade_right"));
            let fade_layer =
                fade_layer.child(edge_gutter_marker(FadeEdge::Right, fade_width, right_dbg));

            let mut viewport = expanded_viewport(viewport_width, fade_width);
            if let Some(ref name) = dbg {
                let sel = format!("{name}_viewport");
                viewport = viewport.debug_selector(move || sel.clone());
            }

            let line_dbg = dbg.as_ref().map(|n| format!("{n}_line"));
            let font_size = self.font_size;
            let font_weight = self.font_weight;
            let text_color = self
                .text_color
                .unwrap_or_else(|| window.text_style().color.into());

            let (char_lens, char_centers) =
                measure_char_geometry(&self.text, font_size, font_weight, text_width, window);

            if self.active && !is_motion_reduced {
                let text = self.text;
                let char_lens_c = char_lens.clone();
                let char_centers_c = char_centers.clone();

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
                                element.child(faded_marquee_line(
                                    text.clone(),
                                    fade_width - shift * progress,
                                    font_size,
                                    font_weight,
                                    text_color,
                                    fade_width,
                                    viewport_width,
                                    true,
                                    &char_lens_c,
                                    &char_centers_c,
                                    line_dbg.clone(),
                                ))
                            },
                        ),
                );
            } else {
                viewport = viewport.child(faded_marquee_line(
                    self.text,
                    fade_width,
                    font_size,
                    font_weight,
                    text_color,
                    fade_width,
                    viewport_width,
                    false,
                    &char_lens,
                    &char_centers,
                    line_dbg,
                ));
            }

            if self.fade_enabled {
                viewport = viewport.child(fade_layer);
            }

            anchor.child(viewport)
        } else {
            let line_dbg = dbg.as_ref().map(|n| format!("{n}_line"));
            anchor.child(static_text_line(
                self.text,
                self.font_size,
                self.font_weight,
                self.text_color,
                line_dbg,
            ))
        }
    }
}

/// Viewport expanded by `fade_width` on both sides so overflowing text dissolves in surrounding gutters.
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

/// Structural marker placed in the gutter for bounds tracking without drawing any background quad.
fn edge_gutter_marker(edge: FadeEdge, width: Pixels, debug_sel: Option<String>) -> Div {
    let mut marker = div().absolute().top_0().bottom_0().w(width);
    if let Some(sel) = debug_sel {
        marker = marker.debug_selector(move || sel.clone());
    }
    match edge {
        FadeEdge::Left => marker.left_0(),
        FadeEdge::Right => marker.right_0(),
    }
}

/// Precomputes character byte lengths and horizontal center positions calibrated to line width.
fn measure_char_geometry(
    text: &SharedString,
    font_size: Pixels,
    font_weight: FontWeight,
    text_width: Pixels,
    window: &mut Window,
) -> (Vec<usize>, Vec<f32>) {
    let mut font = window.text_style().font();
    font.weight = font_weight;

    let char_count = text.chars().count();
    let mut char_lens = Vec::with_capacity(char_count);
    let mut char_centers = Vec::with_capacity(char_count);
    let mut running_x = 0.0f32;

    for c in text.chars() {
        let char_str: SharedString = c.to_string().into();
        let char_run = TextRun {
            len: c.len_utf8(),
            font: font.clone(),
            color: gpui::rgb(0x00ff_ffff).into(),
            ..Default::default()
        };
        let char_shaped = window
            .text_system()
            .shape_line(char_str, font_size, &[char_run], None);
        let char_w = f32::from(char_shaped.width);
        char_centers.push(running_x + char_w * 0.5);
        char_lens.push(c.len_utf8());
        running_x += char_w;
    }

    let scale = if running_x > 0.0 {
        f32::from(text_width) / running_x
    } else {
        1.0
    };
    for center in &mut char_centers {
        *center *= scale;
    }

    (char_lens, char_centers)
}

#[allow(clippy::too_many_arguments)]
fn faded_marquee_line(
    text: SharedString,
    offset: Pixels,
    font_size: Pixels,
    font_weight: FontWeight,
    text_color: Rgba,
    fade_width: Pixels,
    viewport_width: Pixels,
    active: bool,
    char_lens: &[usize],
    char_centers: &[f32],
    debug_name: Option<String>,
) -> Div {
    let fade_w = f32::from(fade_width);
    let view_w = f32::from(viewport_width);
    let offset_f = f32::from(offset);
    let line_height = (font_size * 1.35).max(px(14.0));

    let char_lens_vec = char_lens.to_vec();
    let char_centers_vec = char_centers.to_vec();

    let mut line = div()
        .relative()
        .left(offset)
        .flex_none()
        .h(line_height)
        .w(px(
            char_centers.last().copied().unwrap_or(0.0) + fade_w * 2.0
        ));

    if let Some(name) = debug_name {
        line = line.debug_selector(move || name.clone());
    }

    line.child(canvas(
        move |_bounds, _window, _cx| {},
        move |bounds, (), window, cx| {
            let mut font = window.text_style().font();
            font.weight = font_weight;
            let mut runs = Vec::with_capacity(char_lens_vec.len());

            for (i, &len) in char_lens_vec.iter().enumerate() {
                let center_x = char_centers_vec.get(i).copied().unwrap_or(0.0);
                let pos_x = offset_f + center_x;

                let alpha_left = if active && offset_f < fade_w {
                    if pos_x < fade_w {
                        let t = (pos_x / fade_w).clamp(0.0, 1.0);
                        t * t * (3.0 - 2.0 * t)
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                let gutter_start = fade_w + view_w;
                let alpha_right = if pos_x > gutter_start {
                    let t = (1.0 - (pos_x - gutter_start) / fade_w).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                } else {
                    1.0
                };

                let char_alpha = alpha_left.min(alpha_right);
                runs.push(TextRun {
                    len,
                    font: font.clone(),
                    color: text_color.opacity(char_alpha).into(),
                    ..Default::default()
                });
            }

            let shaped = window
                .text_system()
                .shape_line(text, font_size, &runs, None);
            let _ = shaped.paint(
                bounds.origin,
                line_height,
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            );
        },
    ))
}

fn static_text_line(
    text: SharedString,
    font_size: Pixels,
    font_weight: FontWeight,
    text_color: Option<Rgba>,
    debug_name: Option<String>,
) -> Div {
    let mut line = div()
        .relative()
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

#[cfg(test)]
mod tests;
