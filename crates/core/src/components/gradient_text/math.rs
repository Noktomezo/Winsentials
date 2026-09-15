use std::f32::consts::PI;

use gpui::Rgba;

/// Linearly interpolates between two `Rgba` colors.
#[must_use]
pub fn lerp_color(base: Rgba, target: Rgba, factor: f32) -> Rgba {
    let factor = factor.clamp(0.0, 1.0);
    Rgba {
        r: base.r + (target.r - base.r) * factor,
        g: base.g + (target.g - base.g) * factor,
        b: base.b + (target.b - base.b) * factor,
        a: base.a + (target.a - base.a) * factor,
    }
}

/// Evaluates a multi-stop color gradient along `t in [0.0, 1.0]`.
#[must_use]
pub fn multi_stop_color(stops: &[Rgba], factor: f32) -> Rgba {
    if stops.is_empty() {
        return Rgba::default();
    }
    if stops.len() == 1 {
        return stops[0];
    }

    let factor = factor.clamp(0.0, 1.0);
    let segments = stops.len() - 1;
    let scaled = factor * segments as f32;
    let index = (scaled.floor() as usize).min(segments - 1);
    let local_t = scaled - index as f32;

    lerp_color(stops[index], stops[index + 1], local_t)
}

/// Computes smooth continuous ping-pong progress in `[0.0, 1.0]` with cosine easing at turnaround points.
#[must_use]
pub fn yoyo_progress(delta: f32) -> f32 {
    let delta = delta.clamp(0.0, 1.0);
    let triangle = if delta < 0.5 {
        delta * 2.0
    } else {
        (1.0 - delta) * 2.0
    };
    (1.0 - (triangle * PI).cos()) * 0.5
}

/// Calculates the gradient sample coordinate `[0.0, 1.0]` for character at `index`.
#[must_use]
pub fn character_sample_pos(
    index: usize,
    total_chars: usize,
    progress: f32,
    shift_amplitude: f32,
) -> f32 {
    let u = if total_chars <= 1 {
        0.5
    } else {
        index as f32 / (total_chars - 1) as f32
    };

    let shift = shift_amplitude.clamp(0.0, 0.8);
    (u * (1.0 - shift) + progress * shift).clamp(0.0, 1.0)
}
