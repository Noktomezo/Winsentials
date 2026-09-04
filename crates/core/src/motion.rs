use gpui::{Rgba, SpringAnimation, SpringConfig};

use crate::theme::Theme;

#[must_use]
pub fn lerp_rgba(c1: Rgba, c2: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    Rgba {
        r: c1.r + (c2.r - c1.r) * t,
        g: c1.g + (c2.g - c1.g) * t,
        b: c1.b + (c2.b - c1.b) * t,
        a: c1.a + (c2.a - c1.a) * t,
    }
}

#[must_use]
pub fn lerp_item_bg(accent: Rgba, val: f32) -> Rgba {
    let val = val.clamp(0.0, 1.0);
    Rgba {
        r: accent.r,
        g: accent.g,
        b: accent.b,
        a: val,
    }
}

#[must_use]
pub fn lerp_item_text(theme: &Theme, val: f32) -> Rgba {
    let val = val.clamp(0.0, 1.0);
    if val <= 0.5 {
        let t = val / 0.5;
        lerp_rgba(theme.text_primary, theme.accent_blue, t)
    } else {
        let t = (val - 0.5) / 0.5;
        lerp_rgba(theme.accent_blue, theme.selected_text, t)
    }
}

pub const SPRING_HOVER_STIFFNESS: f32 = 350.0;
pub const SPRING_HOVER_DAMPING: f32 = 28.0;

#[must_use]
pub fn hover_spring(target: f32) -> SpringAnimation<f32> {
    SpringAnimation::new(SpringConfig::new(
        SPRING_HOVER_STIFFNESS,
        SPRING_HOVER_DAMPING,
        1.0,
    ))
    .to(target)
    .with_epsilon(0.005)
}
