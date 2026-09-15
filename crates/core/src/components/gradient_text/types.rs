use std::time::Duration;

use gpui::{Pixels, Rgba, px};

/// Default duration for a complete gradient shimmer cycle.
pub const DEFAULT_GRADIENT_DURATION: Duration = Duration::from_secs(7);

/// Default base font size for the gradient text.
pub const DEFAULT_FONT_SIZE: Pixels = px(28.0);

/// Default letter spacing ratio (30% of font size, matching Figma design).
pub const DEFAULT_LETTER_SPACING_RATIO: f32 = 0.3;

/// Default animation shift amplitude along the gradient.
pub const DEFAULT_SHIFT_AMPLITUDE: f32 = 0.35;

/// Tailwind Blue 400 (`#60A5FA`).
pub const TAILWIND_BLUE_400: Rgba = Rgba {
    r: 96.0 / 255.0,
    g: 165.0 / 255.0,
    b: 250.0 / 255.0,
    a: 1.0,
};

/// Tailwind Blue 600 (`#2563EB`).
pub const TAILWIND_BLUE_600: Rgba = Rgba {
    r: 37.0 / 255.0,
    g: 99.0 / 255.0,
    b: 235.0 / 255.0,
    a: 1.0,
};

/// Flow direction for the gradient shimmer effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GradientDirection {
    Horizontal,
    Vertical,
    #[default]
    Diagonal,
}
