use std::sync::Arc;
use std::time::Instant;

use gpui::{div, px, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Rgba, StatefulInteractiveElement, Styled};

use crate::features::navigation::AppRoute;
use crate::shared::motion::hover_spring;
use crate::shared::theme::Theme;
use crate::shared::ui::history_graph::{render_stepped_history_graph_sized, HistoryGraphPalette};
use crate::shared::ui::IconButton;
use super::inspector::render_fps_bottlenecks_inspector;
use super::state::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_expanded_monitor(
    snapshot: &DevPerfSnapshot,
    current_route: AppRoute,
    current_pos: Point<Pixels>,
    status_color: Rgba,
    on_toggle_minimize: &DevActionCallback,
    on_close: &DevActionCallback,
    on_start_drag: &DevDragCallback,
    on_drag_move: &DevDragMoveCallback,
    end_drag_action: &DevActionCallback,
    on_freeze_telemetry: &DevActionCallback,
    on_chart_anim: &DevActionCallback,
    on_continuous: &DevActionCallback,
    on_hover_control: &Option<HoverControlHandler>,
    theme: &Theme,
) -> impl IntoElement {
    let on_toggle = on_toggle_minimize.clone();
    let on_close = on_close.clone();
    let on_start_drag = on_start_drag.clone();
    let on_drag_move = on_drag_move.clone();
    let end_drag_action = end_drag_action.clone();
    let pos_clone = current_pos;
    let is_dragging = snapshot.is_dragging;

    let ram_str = format!("{:.1} MB", snapshot.displayed_memory_mb);
    let frame_samples = Arc::clone(&snapshot.frame_samples);
    let on_hover_control = on_hover_control.clone();
            div()
                .id("dev_perf_monitor_expanded")
                .absolute()
                .left(current_pos.x)
                .top(current_pos.y)
                .w(px(EXPANDED_WIDTH))
                .rounded(px(10.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_mouse_up(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cx.stop_propagation();
                            cb(window, cx);
                        }
                    }
                })
                .on_mouse_up_out(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cb(window, cx);
                        }
                    }
                })
                .on_click(|_, _, cx| cx.stop_propagation())
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                // Header (Draggable)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_move()
                        .on_mouse_down(MouseButton::Left, {
                            let end_cb = end_drag_action.clone();
                            move |event, window, cx| {
                                cx.stop_propagation();
                                if is_dragging {
                                    end_cb(window, cx);
                                } else {
                                    on_start_drag(event.position, pos_clone, window, cx);
                                }
                            }
                        })
                        .on_mouse_move(move |event, window, cx| {
                            if event.dragging() {
                                cx.stop_propagation();
                                on_drag_move(event.position, true, window, cx);
                            } else if is_dragging {
                                cx.stop_propagation();
                                on_drag_move(event.position, false, window, cx);
                            }
                        })
                        .on_mouse_up(MouseButton::Left, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                cx.stop_propagation();
                                cb(window, cx);
                            }
                        })
                        .on_mouse_up_out(MouseButton::Left, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                if is_dragging {
                                    cb(window, cx);
                                }
                            }
                        })
                        .on_mouse_up(MouseButton::Right, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                cx.stop_propagation();
                                cb(window, cx);
                            }
                        })
                        .on_mouse_up_out(MouseButton::Right, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                if is_dragging {
                                    cb(window, cx);
                                }
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().size(px(8.0)).rounded_full().bg(status_color))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.text_primary)
                                        .child("Dev Profiler HUD"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .px(px(4.0))
                                        .py(px(1.0))
                                        .rounded(px(4.0))
                                        .bg(theme.input_bg)
                                        .border_1()
                                        .border_color(theme.card_border)
                                        .text_color(theme.text_muted)
                                        .child("DEV ONLY"),
                                ),
                        )
                        .child({
                            let is_min_hovered = snapshot.hovered_control == Some("min");
                            let min_spring = hover_spring(if is_min_hovered { 0.5 } else { 0.0 });
                            let on_hover_min = on_hover_control.clone();

                            let is_close_hovered = snapshot.hovered_control == Some("close");
                            let close_spring =
                                hover_spring(if is_close_hovered { 0.6 } else { 0.0 });
                            let on_hover_close = on_hover_control.clone();

                            div()
                                .flex()
                                .items_center()
                                .gap(px(2.0))
                                .child(
                                    IconButton::new("dev_perf_minimize_btn", "icons/minus.svg")
                                        .button_size(px(24.0))
                                        .icon_size(px(14.0))
                                        .spring(min_spring, theme.accent_cyan)
                                        .on_hover(move |hov, window, cx| {
                                            if let Some(ref h) = on_hover_min {
                                                h("min", hov, window, cx);
                                            }
                                        })
                                        .on_mouse_down(move |window, cx| {
                                            on_toggle(window, cx);
                                        }),
                                )
                                .child(
                                    IconButton::new("dev_perf_close_btn", "icons/x.svg")
                                        .button_size(px(24.0))
                                        .icon_size(px(14.0))
                                        .destructive(true)
                                        .spring(close_spring, theme.accent_red)
                                        .on_hover(move |hov, window, cx| {
                                            if let Some(ref h) = on_hover_close {
                                                h("close", hov, window, cx);
                                            }
                                        })
                                        .on_mouse_down(move |window, cx| {
                                            on_close(window, cx);
                                        }),
                                )
                        }),
                )
                // Metrics Hero Block with Monospace Right-Aligned Digits
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .p(px(8.0))
                        .rounded(px(8.0))
                        .bg(theme.input_bg)
                        .border_1()
                        .border_color(theme.input_border)
                        // Top Hero Row: Big FPS + Target status
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_family("Consolas")
                                                .text_2xl()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(status_color)
                                                .child(if snapshot.displayed_fps <= 0.0 {
                                                    "IDLE".to_string()
                                                } else {
                                                    format!("{:.0}", snapshot.displayed_fps)
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme.text_muted)
                                                .child(if snapshot.displayed_fps <= 0.0 {
                                                    ""
                                                } else {
                                                    "FPS"
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_end()
                                        .font_family("Consolas")
                                        .child(
                                            div().text_xs().text_color(theme.text_primary).child(
                                                format!(
                                                    "TIME: {:.1} ms",
                                                    snapshot.displayed_frame_ms
                                                ),
                                            ),
                                        )
                                        .child(div().text_xs().text_color(theme.text_muted).child(
                                            format!(
                                                "P95:  {:.1} ms",
                                                snapshot.displayed_p95_ms
                                            ),
                                        )),
                                ),
                        )
                        // Secondary Stats Row: Drop %, CPU render time, and Memory
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .pt(px(4.0))
                                .border_t_1()
                                .border_color(theme.card_border)
                                .font_family("Consolas")
                                .text_xs()
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("DROP:"))
                                        .child(
                                            div()
                                                .text_color(
                                                    if snapshot.displayed_drop_rate > 0.0 {
                                                        theme.accent_red
                                                    } else {
                                                        theme.accent_green
                                                    },
                                                )
                                                .child(format!(
                                                    "{:.1}%",
                                                    snapshot.displayed_drop_rate
                                                )),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("CPU:"))
                                        .child(
                                            div().text_color(theme.text_primary).child(format!(
                                                "{:.1}ms",
                                                snapshot.cpu_draw_ms
                                            )),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("MEM:"))
                                        .child(div().text_color(theme.text_primary).child(ram_str)),
                                ),
                        ),
                )
                // Frame Time Graph (Reusing shared stepped history graph from hardware pages)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Frame Time History")
                                .child("16.6ms target"),
                        )
                        .child(render_stepped_history_graph_sized(
                            &frame_samples,
                            None,
                            Instant::now(),
                            theme,
                            HistoryGraphPalette::Semantic,
                            "dev_perf_chart_glide",
                            (33.3, 33.3),
                            "ms",
                            px(48.0),
                        )),
                )
                .child(render_fps_bottlenecks_inspector(
                    current_route,
                    snapshot,
                    on_freeze_telemetry,
                    on_chart_anim,
                    on_continuous,
                    &on_hover_control,
                    theme,
                ))
}
