use std::sync::Arc;

use gpui::{
    Bounds, Pixels, Rgba, SharedString, TextAlign, TextRun, TransformationMatrix,
    fill, point, px, size,
};

use crate::theme::Theme;
#[allow(clippy::wildcard_imports)]
use super::math::*;

#[allow(clippy::too_many_arguments, clippy::cast_precision_loss)]
#[allow(clippy::too_many_lines, clippy::ref_option)]
pub(crate) fn draw_history_graph_tooltip(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    bounds: Bounds<Pixels>,
    top: Pixels,
    bottom: Pixels,
    left: Pixels,
    w: Pixels,
    h: Pixels,
    usable_h: Pixels,
    step_w: Pixels,
    smooth_shift: Pixels,
    sub_ratio: f32,
    max_value: f32,
    count: usize,
    history_data: &[f32],
    secondary_history: &Option<(Arc<[f32]>, Rgba)>,
    tooltip_suffix: &SharedString,
    theme: &Theme,
    palette: HistoryGraphPalette,
    theme_text: Rgba,
    theme_input_bg: Rgba,
) {let cursor = window.mouse_position();
                            if bounds.contains(&cursor) && h >= px(60.0) {
                                let cursor_step = (cursor.x - left + smooth_shift) / step_w;
                                let index = stepped_history_index(cursor_step, count);
                                let value = history_data[index].clamp(0.0, max_value);
                                let hover_color =
                                    history_graph_color(value / max_value * 100.0, theme, palette);

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
}
