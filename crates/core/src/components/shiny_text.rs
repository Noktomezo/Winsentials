use std::f32::consts::PI;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, Styled, TextRun, Window, canvas, div,
    px,
};

pub const DEFAULT_SHINY_DURATION: Duration = Duration::from_millis(2600);
pub const DEFAULT_SWEEP_FRACTION: f32 = 0.65;
pub const DEFAULT_SPREAD: Pixels = px(28.0);
pub const DEFAULT_FONT_SIZE: Pixels = px(12.0);

#[derive(IntoElement)]
pub struct ShinyText {
    id: ElementId,
    text: SharedString,
    font_size: Pixels,
    font_weight: FontWeight,
    base_color: Rgba,
    shine_color: Rgba,
    duration: Duration,
    sweep_fraction: f32,
    spread: Pixels,
    disabled: bool,
}

impl ShinyText {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            font_size: DEFAULT_FONT_SIZE,
            font_weight: FontWeight::MEDIUM,
            base_color: gpui::rgb(0x00b5_b5b5),
            shine_color: gpui::rgb(0x00ff_ffff),
            duration: DEFAULT_SHINY_DURATION,
            sweep_fraction: DEFAULT_SWEEP_FRACTION,
            spread: DEFAULT_SPREAD,
            disabled: false,
        }
    }

    #[must_use]
    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    #[must_use]
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    #[must_use]
    pub fn base_color(mut self, color: impl Into<Rgba>) -> Self {
        self.base_color = color.into();
        self
    }

    #[must_use]
    pub fn shine_color(mut self, color: impl Into<Rgba>) -> Self {
        self.shine_color = color.into();
        self
    }

    #[must_use]
    pub fn speed(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub fn sweep_fraction(mut self, fraction: f32) -> Self {
        self.sweep_fraction = fraction.clamp(0.1, 0.95);
        self
    }

    #[must_use]
    pub fn spread(mut self, spread: Pixels) -> Self {
        self.spread = spread;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[must_use]
pub fn lerp_color(base: Rgba, shine: Rgba, factor: f32) -> Rgba {
    let factor = factor.clamp(0.0, 1.0);
    Rgba {
        r: base.r + (shine.r - base.r) * factor,
        g: base.g + (shine.g - base.g) * factor,
        b: base.b + (shine.b - base.b) * factor,
        a: base.a + (shine.a - base.a) * factor,
    }
}

#[must_use]
pub fn shine_intensity(char_center_x: f32, shine_center_x: f32, spread: f32) -> f32 {
    let distance = (char_center_x - shine_center_x).abs();
    if distance >= spread || spread <= f32::EPSILON {
        return 0.0;
    }
    // Smooth bell curve with zero derivatives at edges: cos^2( (dist / spread) * (PI / 2) )
    let normalized = distance / spread;
    let cos_val = (normalized * (PI * 0.5)).cos();
    cos_val * cos_val
}

#[must_use]
pub fn calculate_shine_center(
    progress: f32,
    sweep_fraction: f32,
    text_width: f32,
    spread: f32,
) -> Option<f32> {
    let progress = progress.clamp(0.0, 1.0);
    if progress >= sweep_fraction {
        return None;
    }
    let local_t = progress / sweep_fraction;
    // Smoothstep easing for natural sweep acceleration and deceleration
    let eased = local_t * local_t * (3.0 - 2.0 * local_t);
    let start_x = -spread;
    let end_x = text_width + spread;
    Some(start_x + eased * (end_x - start_x))
}

impl RenderOnce for ShinyText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if cx.reduce_motion() || self.disabled {
            return div()
                .id(self.id)
                .text_size(self.font_size)
                .font_weight(self.font_weight)
                .text_color(self.base_color)
                .whitespace_nowrap()
                .child(self.text)
                .into_any_element();
        }

        let mut font = window.text_style().font();
        font.weight = self.font_weight;

        // Measure overall text bounds
        let base_run = TextRun {
            len: self.text.len(),
            font: font.clone(),
            color: self.base_color.into(),
            ..Default::default()
        };
        let shaped =
            window
                .text_system()
                .shape_line(self.text.clone(), self.font_size, &[base_run], None);
        let text_width = shaped.width;
        let line_height = (self.font_size * 1.35).max(px(14.0));

        // Precompute character centers so animation frame needs no string shaping for layout
        let mut char_offsets = Vec::with_capacity(self.text.chars().count());
        let mut running_x = 0.0;
        for c in self.text.chars() {
            let char_str: SharedString = c.to_string().into();
            let char_run = TextRun {
                len: c.len_utf8(),
                font: font.clone(),
                color: self.base_color.into(),
                ..Default::default()
            };
            let char_shaped =
                window
                    .text_system()
                    .shape_line(char_str, self.font_size, &[char_run], None);
            let char_w = f32::from(char_shaped.width);
            char_offsets.push(running_x + char_w * 0.5);
            running_x += char_w;
        }

        let text = self.text;
        let font_size = self.font_size;
        let base_color = self.base_color;
        let shine_color = self.shine_color;
        let sweep_fraction = self.sweep_fraction;
        let spread_f32 = f32::from(self.spread);
        let total_w_f32 = running_x.max(f32::from(text_width));

        div()
            .id(self.id.clone())
            .w(text_width)
            .h(line_height)
            .with_animation(
                ElementId::Name(format!("{:?}_shine", self.id).into()),
                Animation::new(self.duration).repeat(),
                move |container, delta| {
                    let maybe_shine =
                        calculate_shine_center(delta, sweep_fraction, total_w_f32, spread_f32);
                    let font_c = font.clone();
                    let text_c = text.clone();
                    let offsets = char_offsets.clone();

                    container.child(canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, (), window, cx| {
                            let runs: Vec<TextRun> = if let Some(shine_x) = maybe_shine {
                                let mut runs = Vec::with_capacity(offsets.len());
                                for (i, c) in text_c.chars().enumerate() {
                                    let char_x = offsets.get(i).copied().unwrap_or(0.0);
                                    let intensity = shine_intensity(char_x, shine_x, spread_f32);
                                    let col = lerp_color(base_color, shine_color, intensity);
                                    runs.push(TextRun {
                                        len: c.len_utf8(),
                                        font: font_c.clone(),
                                        color: col.into(),
                                        ..Default::default()
                                    });
                                }
                                runs
                            } else {
                                vec![TextRun {
                                    len: text_c.len(),
                                    font: font_c.clone(),
                                    color: base_color.into(),
                                    ..Default::default()
                                }]
                            };

                            let line = window.text_system().shape_line(
                                text_c.clone(),
                                font_size,
                                &runs,
                                None,
                            );
                            if let Err(error) = line.paint(
                                bounds.origin,
                                line_height,
                                gpui::TextAlign::Left,
                                None,
                                window,
                                cx,
                            ) {
                                eprintln!("failed to paint shiny text: {error:?}");
                            }
                        },
                    ))
                },
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_color() {
        let base = gpui::rgb(0x0000_0000);
        let shine = gpui::rgb(0x00ff_ffff);

        let c0 = lerp_color(base, shine, 0.0);
        assert!((c0.r - 0.0).abs() < f32::EPSILON);

        let c1 = lerp_color(base, shine, 1.0);
        assert!((c1.r - 1.0).abs() < f32::EPSILON);

        let c_half = lerp_color(base, shine, 0.5);
        assert!((c_half.r - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shine_intensity_bounds() {
        let spread = 30.0;
        let center = 50.0;

        // Peak at center
        assert!((shine_intensity(50.0, center, spread) - 1.0).abs() < 1e-5);

        // Zero outside spread
        assert_eq!(shine_intensity(90.0, center, spread), 0.0);
        assert_eq!(shine_intensity(10.0, center, spread), 0.0);

        // Smoothly positive within spread
        let mid = shine_intensity(65.0, center, spread);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_calculate_shine_center_pause() {
        let sweep_fraction = 0.65;
        let text_w = 100.0;
        let spread = 20.0;

        // Active sweep phase
        let start = calculate_shine_center(0.0, sweep_fraction, text_w, spread);
        assert!(start.is_some());
        assert!((start.unwrap() - (-spread)).abs() < 1e-4);

        let mid = calculate_shine_center(0.325, sweep_fraction, text_w, spread);
        assert!(mid.is_some());
        assert!(mid.unwrap() > -spread && mid.unwrap() < text_w + spread);

        // Pause phase: returns None
        assert!(calculate_shine_center(0.70, sweep_fraction, text_w, spread).is_none());
        assert!(calculate_shine_center(1.00, sweep_fraction, text_w, spread).is_none());
    }

    #[test]
    fn test_shiny_text_builder() {
        let st = ShinyText::new("test_shine", "Activated")
            .font_size(px(14.0))
            .font_weight(FontWeight::BOLD)
            .base_color(gpui::rgb(0x0010_b981))
            .shine_color(gpui::rgb(0x00ff_ffff))
            .speed(Duration::from_secs(3))
            .spread(px(35.0))
            .sweep_fraction(0.7)
            .disabled(true);

        assert_eq!(st.font_size, px(14.0));
        assert_eq!(st.font_weight, FontWeight::BOLD);
        assert_eq!(st.spread, px(35.0));
        assert!((st.sweep_fraction - 0.7).abs() < 1e-5);
        assert!(st.disabled);
    }
}
