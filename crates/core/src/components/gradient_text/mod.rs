use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, Styled, Window, div,
};

pub mod math;
pub mod types;

#[cfg(test)]
mod tests;

pub use math::*;
pub use types::*;

/// Production-grade animated `GradientText` component with custom font, letter-spacing,
/// and smooth diagonal color flow.
#[derive(IntoElement)]
pub struct GradientText {
    id: ElementId,
    text: SharedString,
    font_family: Option<SharedString>,
    font_size: Pixels,
    font_weight: FontWeight,
    letter_spacing: Option<Pixels>,
    colors: Vec<Rgba>,
    duration: Duration,
    shift_amplitude: f32,
    direction: GradientDirection,
    yoyo: bool,
    animated: bool,
    debug_selector: Option<SharedString>,
}

impl GradientText {
    /// Creates a new `GradientText` with default Blue 400 to Blue 600 diagonal shimmer.
    #[must_use]
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            font_family: None,
            font_size: DEFAULT_FONT_SIZE,
            font_weight: FontWeight::NORMAL,
            letter_spacing: None,
            colors: vec![TAILWIND_BLUE_400, TAILWIND_BLUE_600],
            duration: DEFAULT_GRADIENT_DURATION,
            shift_amplitude: DEFAULT_SHIFT_AMPLITUDE,
            direction: GradientDirection::Diagonal,
            yoyo: true,
            animated: true,
            debug_selector: None,
        }
    }

    /// Sets the custom font family.
    #[must_use]
    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// Sets the font size.
    #[must_use]
    pub const fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the font weight.
    #[must_use]
    pub const fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    /// Sets explicit pixel letter spacing between glyphs.
    #[must_use]
    pub const fn letter_spacing(mut self, spacing: Pixels) -> Self {
        self.letter_spacing = Some(spacing);
        self
    }

    /// Sets the color sequence for the gradient stops.
    #[must_use]
    pub fn colors(mut self, colors: Vec<Rgba>) -> Self {
        if !colors.is_empty() {
            self.colors = colors;
        }
        self
    }

    /// Sets the duration of a full animation cycle.
    #[must_use]
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the shift amplitude along the gradient (0.0 to 0.8).
    #[must_use]
    pub fn shift_amplitude(mut self, amplitude: f32) -> Self {
        self.shift_amplitude = amplitude.clamp(0.0, 0.8);
        self
    }

    /// Sets the gradient flow direction.
    #[must_use]
    pub const fn direction(mut self, direction: GradientDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Toggles yoyo (ping-pong) mode.
    #[must_use]
    pub const fn yoyo(mut self, yoyo: bool) -> Self {
        self.yoyo = yoyo;
        self
    }

    /// Enables or disables the animation loop.
    #[must_use]
    pub const fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Sets an optional debug selector for testing.
    #[must_use]
    pub fn debug_selector(mut self, selector: impl Into<SharedString>) -> Self {
        self.debug_selector = Some(selector.into());
        self
    }
}

impl RenderOnce for GradientText {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_reduced_motion = cx.reduce_motion();
        let chars: Vec<char> = self.text.chars().collect();
        let total_chars = chars.len();
        let spacing = self
            .letter_spacing
            .unwrap_or(self.font_size * DEFAULT_LETTER_SPACING_RATIO);
        let debug_sel = self.debug_selector.clone();

        if !self.animated || is_reduced_motion {
            let progress = 0.5;
            let container = make_container(self.id, spacing, debug_sel);

            return container
                .children(chars.into_iter().enumerate().map(|(i, c)| {
                    let sample_t =
                        character_sample_pos(i, total_chars, progress, self.shift_amplitude);
                    let color = multi_stop_color(&self.colors, sample_t);
                    render_char_element(
                        c,
                        color,
                        &self.font_family,
                        self.font_size,
                        self.font_weight,
                    )
                }))
                .into_any_element();
        }

        let anim_id = ElementId::Name(format!("{:?}_anim", self.id).into());
        let font_family = self.font_family.clone();
        let font_size = self.font_size;
        let font_weight = self.font_weight;
        let colors = self.colors.clone();
        let shift_amplitude = self.shift_amplitude;
        let yoyo = self.yoyo;

        let container = make_container(self.id, spacing, debug_sel);

        container
            .with_animation(
                anim_id,
                Animation::new(self.duration).repeat(),
                move |container, delta| {
                    let progress = if yoyo { yoyo_progress(delta) } else { delta };
                    container.children(chars.iter().enumerate().map(|(i, &c)| {
                        let sample_t =
                            character_sample_pos(i, total_chars, progress, shift_amplitude);
                        let color = multi_stop_color(&colors, sample_t);
                        render_char_element(c, color, &font_family, font_size, font_weight)
                    }))
                },
            )
            .into_any_element()
    }
}

fn make_container(
    id: ElementId,
    spacing: Pixels,
    debug_sel: Option<SharedString>,
) -> gpui::Stateful<gpui::Div> {
    let mut container = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap(spacing);

    if let Some(sel) = debug_sel {
        container = container.debug_selector(move || sel.to_string());
    }

    container
}

fn render_char_element(
    c: char,
    color: Rgba,
    font_family: &Option<SharedString>,
    font_size: Pixels,
    font_weight: FontWeight,
) -> impl IntoElement {
    let mut el = div()
        .text_size(font_size)
        .font_weight(font_weight)
        .text_color(color)
        .whitespace_nowrap()
        .flex_none()
        .child(c.to_string());

    if let Some(family) = font_family {
        el = el.font_family(family.clone());
    }

    el
}
