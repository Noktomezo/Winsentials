use gpui::{div, px, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, Rgba, StatefulInteractiveElement, Styled};

use crate::shared::motion::hover_spring;
use crate::shared::theme::Theme;
use crate::shared::ui::IconButton;
use super::state::*;

pub(crate) fn render_minimized_monitor(
    snapshot: &DevPerfSnapshot,
    current_pos: Point<Pixels>,
    status_color: Rgba,
    fps_text: &str,
    on_toggle_minimize: &DevActionCallback,
    on_start_drag: &DevDragCallback,
    on_drag_move: &DevDragMoveCallback,
    end_drag_action: &DevActionCallback,
    on_hover_control: &Option<HoverControlHandler>,
    theme: &Theme,
) -> impl IntoElement {
    let on_toggle = on_toggle_minimize.clone();
    let on_start_drag = on_start_drag.clone();
    let on_drag_move = on_drag_move.clone();
    let end_drag_action = end_drag_action.clone();
    let pos_clone = current_pos;
    let is_dragging = snapshot.is_dragging;

    let is_expand_hovered = snapshot.hovered_control == Some("expand");
    let expand_spring = hover_spring(if is_expand_hovered { 0.5 } else { 0.0 });
    let on_hover_expand = on_hover_control.clone();
            div()
                .id("dev_perf_monitor_minimized")
                .absolute()
                .left(current_pos.x)
                .top(current_pos.y)
                .w(px(MINIMIZED_WIDTH))
                .h(px(MINIMIZED_HEIGHT))
                .rounded(px(8.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.0))
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
                .on_mouse_down(MouseButton::Right, {
                    let end_cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        cx.stop_propagation();
                        if is_dragging {
                            end_cb(window, cx);
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
                .on_click(|_, _, cx| cx.stop_propagation())
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().size(px(8.0)).rounded_full().bg(status_color))
                        .child(
                            div()
                                .font_family("Consolas")
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(fps_text.to_string()),
                        )
                        .child(
                            div()
                                .font_family("Consolas")
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text_muted)
                                .child(format!("{:.1}ms", snapshot.displayed_frame_ms)),
                        ),
                )
                .child(
                    IconButton::new("dev_perf_expand_btn", "icons/maximize-2.svg")
                        .button_size(px(22.0))
                        .icon_size(px(13.0))
                        .spring(expand_spring, theme.accent_cyan)
                        .on_hover(move |hov, window, cx| {
                            if let Some(ref h) = on_hover_expand {
                                h("expand", hov, window, cx);
                            }
                        })
                        .on_mouse_down(move |window, cx| {
                            on_toggle(window, cx);
                        }),
                )
}
