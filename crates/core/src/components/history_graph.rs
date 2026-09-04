use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, Background, Bounds, ContentMask, Interpolate, IntoElement,
    ParentElement, PathBuilder, Pixels, Rgba, SharedString, Styled, TextAlign, TextRun,
    TransformationMatrix, canvas, div, fill, linear_color_stop, linear_gradient, point,
    pulsating_between, px, size,
};

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

#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn render_stepped_history_graph(
    history: &[f32],
    secondary_history: Option<(&[f32], Rgba)>,
    sample_instant: std::time::Instant,
    theme: &Theme,
    palette: HistoryGraphPalette,
    animation_id: &'static str,
    max_scale: (f32, f32),
    tooltip_suffix: impl Into<SharedString>,
) -> impl IntoElement {
    render_stepped_history_graph_sized(
        history,
        secondary_history,
        sample_instant,
        theme,
        palette,
        animation_id,
        max_scale,
        tooltip_suffix,
        px(140.0),
    )
}

#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn render_stepped_history_graph_sized(
    history: &[f32],
    secondary_history: Option<(&[f32], Rgba)>,
    sample_instant: std::time::Instant,
    theme: &Theme,
    palette: HistoryGraphPalette,
    animation_id: &'static str,
    max_scale: (f32, f32),
    tooltip_suffix: impl Into<SharedString>,
    graph_height: Pixels,
) -> impl IntoElement {
    let (prev_scale, target_scale) = max_scale;
    let prev_scale = prev_scale.max(f32::EPSILON);
    let target_scale = target_scale.max(f32::EPSILON);
    let history_data: Arc<[f32]> = if history.is_empty() {
        Arc::from([0.0, 0.0])
    } else {
        Arc::from(history)
    };
    let secondary_history: Option<(Arc<[f32]>, Rgba)> =
        secondary_history.map(|(hist, color)| (Arc::from(hist), color));
    let theme = *theme;
    let theme_input_bg = theme.input_bg;
    let theme_input_border = theme.input_border;
    let theme_grid_color = theme.card_border;
    let theme_text = theme.text_primary;
    let tooltip_suffix = tooltip_suffix.into();
    let color_bands: &'static [(f32, f32)] = match palette {
        HistoryGraphPalette::Semantic => &GRAPH_COLOR_BANDS,
        HistoryGraphPalette::Solid(_) => &SOLID_COLOR_BANDS,
    };

    div()
        .h(graph_height)
        .w_full()
        .rounded(px(8.0))
        .bg(theme_input_bg)
        .border_1()
        .border_color(theme_input_border)
        .overflow_hidden()
        .with_animation(
            animation_id,
            Animation::new(Duration::from_millis(1_400))
                .repeat()
                .with_easing(pulsating_between(0.0, 1.0)),
            move |graph, pulse| {
                let history_data = Arc::clone(&history_data);
                let secondary_history = secondary_history.clone();
                let tooltip_suffix = tooltip_suffix.clone();
                graph.child(
                    canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, (), window, cx| {
                            let w = bounds.size.width;
                            let h = bounds.size.height;
                            if w <= px(0.0) || h <= px(0.0) {
                                return;
                            }
                            let left = bounds.origin.x;
                            let top = bounds.origin.y;
                            let pad_y = px(4.0).min(h / 6.0);
                            let bottom = top + h - pad_y;
                            let usable_h = (h - (pad_y * 2.0)).max(px(1.0));

                            // 1. Background grid lines (25%, 50%, 75%)
                            for ratio in [0.25, 0.50, 0.75] {
                                let y = bottom - (usable_h * ratio);
                                window.paint_quad(fill(
                                    Bounds {
                                        origin: point(left, y),
                                        size: size(w, px(1.0)),
                                    },
                                    theme_grid_color.opacity(0.35),
                                ));
                            }

                            // 2. Smooth conveyor glide anchored to sample instant
                            let elapsed_secs = sample_instant.elapsed().as_secs_f32();
                            let sub_ratio = (elapsed_secs / 0.5).clamp(0.0, 1.0);
                            let scale_progress = sub_ratio * sub_ratio * (3.0 - 2.0 * sub_ratio);
                            let max_value = (prev_scale
                                + (target_scale - prev_scale) * scale_progress)
                                .max(f32::EPSILON);

                            let count = history_data.len().max(2);
                            let step_w = w / (count as f32 - 1.0);
                            let smooth_shift = step_w * sub_ratio;
                            let first_y = bottom
                                - (usable_h * (history_data[0].clamp(0.0, max_value) / max_value));
                            let mut line_path = PathBuilder::stroke(px(2.0));
                            line_path.move_to(point(left - smooth_shift, first_y));

                            for i in 0..count {
                                let val = history_data[i].clamp(0.0, max_value);
                                let percent = val / max_value * 100.0;
                                let x0 = left + (step_w * i as f32) - smooth_shift;
                                let x1 = x0 + step_w;
                                let y_curr = bottom - (usable_h * (val / max_value));

                                let draw_x0 = x0.max(left);
                                let draw_x1 = x1.min(left + w);

                                if draw_x1 > draw_x0 {
                                    let seg_w = draw_x1 - draw_x0;
                                    for (lower, upper) in color_bands.iter().copied() {
                                        let clipped_upper = percent.min(upper);
                                        if clipped_upper <= lower {
                                            continue;
                                        }

                                        let band_top = bottom - usable_h * (clipped_upper / 100.0);
                                        let band_bottom = bottom - usable_h * (lower / 100.0);
                                        let from = history_graph_color(lower, &theme, palette)
                                            .opacity(0.12);
                                        let to =
                                            history_graph_color(clipped_upper, &theme, palette)
                                                .opacity(0.12);
                                        let background: Background = if from == to {
                                            from.into()
                                        } else {
                                            linear_gradient(
                                                0.0,
                                                linear_color_stop(from, 0.0),
                                                linear_color_stop(to, 1.0),
                                            )
                                        };
                                        window.paint_quad(fill(
                                            Bounds {
                                                origin: point(draw_x0, band_top),
                                                size: size(seg_w, band_bottom - band_top),
                                            },
                                            background,
                                        ));
                                    }
                                }

                                if i + 1 < count {
                                    let next_val = history_data[i + 1].clamp(0.0, max_value);
                                    let y_next = bottom - (usable_h * (next_val / max_value));
                                    let radius = stepped_corner_radius(step_w, y_curr, y_next);

                                    if radius.height == px(0.0) {
                                        line_path.line_to(point(x1, y_curr));
                                    } else {
                                        let direction = if y_next > y_curr { 1.0 } else { -1.0 };
                                        line_path.line_to(point(x1 - radius.width, y_curr));
                                        line_path.curve_to(
                                            point(x1, y_curr + radius.height * direction),
                                            point(x1, y_curr),
                                        );
                                        line_path
                                            .line_to(point(x1, y_next - radius.height * direction));
                                        line_path.curve_to(
                                            point(x1 + radius.width, y_next),
                                            point(x1, y_next),
                                        );
                                    }
                                } else {
                                    line_path.line_to(point(x1, y_curr));
                                }
                            }

                            let line_path = line_path
                                .build()
                                .expect("Graph path geometry must be valid");
                            let path_bottom =
                                line_path.bounds.origin.y + line_path.bounds.size.height;
                            let path_low =
                                ((bottom - path_bottom) / usable_h * 100.0).clamp(0.0, 100.0);
                            let path_high = ((bottom - line_path.bounds.origin.y) / usable_h
                                * 100.0)
                                .clamp(0.0, 100.0);
                            let path_span = (path_high - path_low).max(f32::EPSILON);

                            for (lower, upper) in color_bands.iter().copied() {
                                let clipped_lower = lower.max(path_low);
                                let clipped_upper = upper.min(path_high);
                                if clipped_upper <= clipped_lower {
                                    continue;
                                }

                                let zone_top = bottom - usable_h * (clipped_upper / 100.0);
                                let zone_bottom = bottom - usable_h * (clipped_lower / 100.0);
                                let from = history_graph_color(clipped_lower, &theme, palette);
                                let to = history_graph_color(clipped_upper, &theme, palette);
                                let background: Background = if from == to {
                                    from.into()
                                } else {
                                    linear_gradient(
                                        0.0,
                                        linear_color_stop(
                                            from,
                                            (clipped_lower - path_low) / path_span,
                                        ),
                                        linear_color_stop(
                                            to,
                                            (clipped_upper - path_low) / path_span,
                                        ),
                                    )
                                };
                                window.with_content_mask(
                                    Some(ContentMask {
                                        bounds: Bounds {
                                            origin: point(left, zone_top),
                                            size: size(w, zone_bottom - zone_top),
                                        },
                                    }),
                                    |window| window.paint_path(line_path.clone(), background),
                                );
                            }

                            let live_value = history_data[count - 1].clamp(0.0, max_value);
                            let live_percent = live_value / max_value * 100.0;
                            let live_color = history_graph_color(live_percent, &theme, palette);
                            let live_x = left + w - px(1.0);
                            let live_y = bottom - (usable_h * (live_value / max_value));
                            let halo_size = px(6.0 + 6.0 * pulse);
                            window.paint_quad(
                                fill(
                                    Bounds {
                                        origin: point(
                                            live_x - halo_size / 2.0,
                                            live_y - halo_size / 2.0,
                                        ),
                                        size: size(halo_size, halo_size),
                                    },
                                    live_color.opacity(0.28 * (1.0 - pulse)),
                                )
                                .corner_radii(halo_size / 2.0),
                            );
                            window.paint_quad(
                                fill(
                                    Bounds {
                                        origin: point(live_x - px(3.0), live_y - px(3.0)),
                                        size: size(px(6.0), px(6.0)),
                                    },
                                    live_color,
                                )
                                .corner_radii(px(3.0)),
                            );

                            if let Some((secondary_history, color)) = &secondary_history {
                                let secondary_count = secondary_history.len().max(2);
                                let secondary_step = w / (secondary_count as f32 - 1.0);
                                let secondary_shift = secondary_step * sub_ratio;
                                let first = secondary_history[0].clamp(0.0, max_value);
                                let mut path = PathBuilder::stroke(px(1.5));
                                path.move_to(point(
                                    left - secondary_shift,
                                    bottom - usable_h * (first / max_value),
                                ));

                                for index in 0..secondary_count {
                                    let value = secondary_history[index].clamp(0.0, max_value);
                                    let next_x = left + secondary_step * (index + 1) as f32
                                        - secondary_shift;
                                    let y = bottom - usable_h * (value / max_value);

                                    if index + 1 < secondary_count {
                                        let next =
                                            secondary_history[index + 1].clamp(0.0, max_value);
                                        let next_y = bottom - usable_h * (next / max_value);
                                        let radius =
                                            stepped_corner_radius(secondary_step, y, next_y);
                                        let direction = if next_y > y { 1.0 } else { -1.0 };
                                        path.line_to(point(next_x - radius.width, y));
                                        path.curve_to(
                                            point(next_x, y + radius.height * direction),
                                            point(next_x, y),
                                        );
                                        path.line_to(point(
                                            next_x,
                                            next_y - radius.height * direction,
                                        ));
                                        path.curve_to(
                                            point(next_x + radius.width, next_y),
                                            point(next_x, next_y),
                                        );
                                    } else {
                                        path.line_to(point(next_x, y));
                                    }
                                }

                                let path = path
                                    .build()
                                    .expect("Secondary graph path geometry must be valid");
                                window.paint_path(path, *color);

                                let live =
                                    secondary_history[secondary_count - 1].clamp(0.0, max_value);
                                let live_y = bottom - usable_h * (live / max_value);
                                window.paint_quad(
                                    fill(
                                        Bounds {
                                            origin: point(live_x - px(2.5), live_y - px(2.5)),
                                            size: size(px(5.0), px(5.0)),
                                        },
                                        *color,
                                    )
                                    .corner_radii(px(2.5)),
                                );
                            }

                            let cursor = window.mouse_position();
                            if bounds.contains(&cursor) && h >= px(60.0) {
                                let cursor_step = (cursor.x - left + smooth_shift) / step_w;
                                let index = stepped_history_index(cursor_step, count);
                                let value = history_data[index].clamp(0.0, max_value);
                                let hover_color =
                                    history_graph_color(value / max_value * 100.0, &theme, palette);

                                window.paint_quad(fill(
                                    Bounds {
                                        origin: point(cursor.x - px(0.5), top),
                                        size: size(px(1.0), h),
                                    },
                                    theme_text.opacity(0.25),
                                ));

                                let has_secondary = secondary_history.is_some();
                                let tooltip_size = size(
                                    px(if tooltip_suffix.as_ref() == "%" {
                                        44.0
                                    } else {
                                        76.0
                                    } + if has_secondary { 18.0 } else { 0.0 }),
                                    px(24.0),
                                );
                                let gap = px(6.0);
                                let tooltip_x = if cursor.x + gap + tooltip_size.width <= left + w {
                                    cursor.x + gap
                                } else {
                                    cursor.x - gap - tooltip_size.width
                                };

                                let secondary_hover =
                                    secondary_history
                                        .as_ref()
                                        .map(|(secondary_history, color)| {
                                            let secondary_count = secondary_history.len().max(2);
                                            let secondary_step = w / (secondary_count as f32 - 1.0);
                                            let secondary_cursor_step = (cursor.x - left
                                                + secondary_step * sub_ratio)
                                                / secondary_step;
                                            let secondary_index = stepped_history_index(
                                                secondary_cursor_step,
                                                secondary_count,
                                            );
                                            (
                                                secondary_history[secondary_index]
                                                    .clamp(0.0, max_value),
                                                *color,
                                            )
                                        });
                                let tooltip_count =
                                    if secondary_hover.is_some() { 2.0 } else { 1.0 };
                                let tooltip_stack_height = tooltip_size.height * tooltip_count
                                    + gap * (tooltip_count - 1.0);
                                let tooltip_stack_y = (cursor.y - tooltip_stack_height / 2.0)
                                    .max(top + gap)
                                    .min(top + h - gap - tooltip_stack_height);

                                for (tooltip_index, (value, color)) in
                                    std::iter::once((value, hover_color))
                                        .chain(secondary_hover)
                                        .enumerate()
                                {
                                    let point_y = bottom - (usable_h * (value / max_value));
                                    window.paint_quad(
                                        fill(
                                            Bounds {
                                                origin: point(
                                                    cursor.x - px(4.0),
                                                    point_y - px(4.0),
                                                ),
                                                size: size(px(8.0), px(8.0)),
                                            },
                                            color,
                                        )
                                        .corner_radii(px(4.0)),
                                    );

                                    let tooltip_y = tooltip_stack_y
                                        + (tooltip_size.height + gap) * tooltip_index as f32;
                                    let tooltip_bounds = Bounds {
                                        origin: point(tooltip_x, tooltip_y),
                                        size: tooltip_size,
                                    };
                                    window.paint_quad(
                                        fill(tooltip_bounds, color.opacity(0.75))
                                            .corner_radii(px(6.0)),
                                    );
                                    window.paint_quad(
                                        fill(
                                            Bounds {
                                                origin: tooltip_bounds.origin
                                                    + point(px(1.0), px(1.0)),
                                                size: tooltip_bounds.size - size(px(2.0), px(2.0)),
                                            },
                                            theme_input_bg,
                                        )
                                        .corner_radii(px(5.0)),
                                    );

                                    let label: SharedString = if tooltip_suffix.as_ref() == "%" {
                                        format!("{value:.0}%").into()
                                    } else {
                                        format!("{value:.1}{tooltip_suffix}").into()
                                    };
                                    let text_run = TextRun {
                                        len: label.len(),
                                        font: window.text_style().font(),
                                        color: color.into(),
                                        ..Default::default()
                                    };
                                    let line = window.text_system().shape_line(
                                        label,
                                        px(11.0),
                                        &[text_run],
                                        None,
                                    );
                                    let line_height = px(14.0);
                                    let icon_path = if has_secondary {
                                        Some(if tooltip_index == 0 {
                                            "icons/arrow-down.svg"
                                        } else {
                                            "icons/arrow-up.svg"
                                        })
                                    } else {
                                        None
                                    };
                                    let text_x = if let Some(icon_path) = icon_path {
                                        let icon_size = px(12.0);
                                        let icon_gap = px(4.0);
                                        let content_width = icon_size + icon_gap + line.width;
                                        let icon_x =
                                            tooltip_x + (tooltip_size.width - content_width) / 2.0;
                                        window
                                            .paint_svg(
                                                Bounds {
                                                    origin: point(
                                                        icon_x,
                                                        tooltip_y
                                                            + (tooltip_size.height - icon_size)
                                                                / 2.0,
                                                    ),
                                                    size: size(icon_size, icon_size),
                                                },
                                                icon_path.into(),
                                                None,
                                                TransformationMatrix::default(),
                                                color.into(),
                                                cx,
                                            )
                                            .expect("Graph tooltip arrow paint failed");
                                        icon_x + icon_size + icon_gap
                                    } else {
                                        tooltip_x + (tooltip_size.width - line.width) / 2.0
                                    };
                                    line.paint(
                                        point(
                                            text_x,
                                            tooltip_y + (tooltip_size.height - line_height) / 2.0,
                                        ),
                                        line_height,
                                        TextAlign::Left,
                                        None,
                                        window,
                                        cx,
                                    )
                                    .expect("Graph tooltip text paint failed");
                                }
                            }
                        },
                    )
                    .size_full(),
                )
            },
        )
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::{
        graph_percent_color, smooth_percent_transition, stepped_corner_radius,
        stepped_history_index,
    };
    use crate::theme::Theme;

    #[test]
    fn graph_colors_blend_between_dashboard_zones() {
        let theme = Theme::dark();

        assert_eq!(graph_percent_color(20.0, &theme), theme.accent_green);
        assert_eq!(graph_percent_color(70.0, &theme), theme.accent_yellow);
        assert_eq!(graph_percent_color(95.0, &theme), theme.accent_red);
    }

    #[test]
    fn maps_cursor_to_stepped_history_segment() {
        assert_eq!(stepped_history_index(0.2, 5), 0);
        assert_eq!(stepped_history_index(2.0, 5), 1);
        assert_eq!(stepped_history_index(4.8, 5), 4);
        assert_eq!(stepped_history_index(9.0, 5), 4);
    }

    #[test]
    fn core_transition_is_smooth_and_clamped() {
        assert_eq!(smooth_percent_transition(10.0, 50.0, -1.0), 10.0);
        assert_eq!(smooth_percent_transition(10.0, 50.0, 0.5), 30.0);
        assert_eq!(smooth_percent_transition(10.0, 50.0, 2.0), 50.0);
    }

    #[test]
    fn rounded_step_radius_stays_inside_short_transitions() {
        let radius = stepped_corner_radius(px(3.0), px(10.0), px(12.0));
        assert!(radius.width <= px(1.5));
        assert!(radius.height <= px(1.0));
    }
}
