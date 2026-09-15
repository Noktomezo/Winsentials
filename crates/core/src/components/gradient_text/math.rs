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

/// Smooth continuous cosine blend between two colors in a seamless loop where `phase in [0.0, 1.0]`.
/// Phase 0.0 corresponds to `c1`, phase 0.5 corresponds to `c2`, and phase 1.0 wraps seamlessly back to `c1`.
#[must_use]
pub fn seamless_two_stop_color(c1: Rgba, c2: Rgba, phase: f32) -> Rgba {
    let phase = phase.rem_euclid(1.0);
    let t = (1.0 - (phase * 2.0 * PI).cos()) * 0.5;
    lerp_color(c1, c2, t)
}

/// Evaluates a multi-stop color gradient along a periodic seamless loop where `phase in [0.0, 1.0]`.
#[must_use]
pub fn seamless_multi_stop_color(stops: &[Rgba], phase: f32) -> Rgba {
    if stops.is_empty() {
        return Rgba::default();
    }
    if stops.len() == 1 {
        return stops[0];
    }
    if stops.len() == 2 {
        return seamless_two_stop_color(stops[0], stops[1], phase);
    }

    let phase = phase.rem_euclid(1.0);
    let count = stops.len();
    let scaled = phase * count as f32;
    let index = (scaled.floor() as usize) % count;
    let next_index = (index + 1) % count;
    let local_t = scaled - scaled.floor();

    lerp_color(stops[index], stops[next_index], local_t)
}

/// Calculates the seamless phase coordinate `[0.0, 1.0)` for character at `index` given animation `progress in [0.0, 1.0]`.
#[must_use]
pub fn character_seamless_phase(index: usize, total_chars: usize, progress: f32) -> f32 {
    let u = if total_chars <= 1 {
        0.0
    } else {
        index as f32 / (2.0 * (total_chars - 1) as f32)
    };

    (u - progress).rem_euclid(1.0)
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
