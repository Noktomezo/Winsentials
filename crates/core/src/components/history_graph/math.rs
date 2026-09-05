use gpui::{Interpolate, Pixels, Rgba, px, size};

use crate::theme::Theme;

pub const GRAPH_COLOR_BANDS: [(f32, f32); 5] = [
    (0.0, 55.0),
    (55.0, 65.0),
    (65.0, 80.0),
    (80.0, 90.0),
    (90.0, 100.0),
];

pub const SOLID_COLOR_BANDS: [(f32, f32); 1] = [(0.0, 100.0)];

#[derive(Clone, Copy)]
pub enum HistoryGraphPalette {
    Semantic,
    Solid(Rgba),
}

#[must_use]
pub fn graph_percent_color(percent: f32, theme: &Theme) -> Rgba {
    if percent < 55.0 {
        theme.accent_green
    } else if percent < 65.0 {
        Rgba::interpolate(
            theme.accent_green,
            theme.accent_yellow,
            (percent - 55.0) / 10.0,
        )
    } else if percent < 80.0 {
        theme.accent_yellow
    } else if percent < 90.0 {
        Rgba::interpolate(
            theme.accent_yellow,
            theme.accent_red,
            (percent - 80.0) / 10.0,
        )
    } else {
        theme.accent_red
    }
}

#[must_use]
pub fn history_graph_color(percent: f32, theme: &Theme, palette: HistoryGraphPalette) -> Rgba {
    match palette {
        HistoryGraphPalette::Semantic => graph_percent_color(percent, theme),
        HistoryGraphPalette::Solid(color) => color,
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn stepped_history_index(cursor_step: f32, count: usize) -> usize {
    (0..count)
        .position(|index| cursor_step <= (index + 1) as f32)
        .unwrap_or_else(|| count.saturating_sub(1))
}

#[must_use]
pub fn stepped_corner_radius(
    step_width: Pixels,
    y_curr: Pixels,
    y_next: Pixels,
) -> gpui::Size<Pixels> {
    let y_min = y_curr.min(y_next);
    let height = y_curr.max(y_next) - y_min;
    size(px(4.0).min(step_width / 2.0), px(4.0).min(height / 2.0))
}

#[must_use]
pub fn smooth_percent_transition(from: f32, to: f32, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let eased = progress * progress * (3.0 - 2.0 * progress);
    from + (to - from) * eased
}