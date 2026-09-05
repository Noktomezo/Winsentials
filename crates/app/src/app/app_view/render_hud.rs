use gpui::{Context, Div, Stateful, InteractiveElement, ParentElement, Styled, Window, div, px};

#[cfg(debug_assertions)]
use std::sync::Arc;
#[cfg(debug_assertions)]
use gpui::{MouseButton, Pixels, Point};

use super::AppView;

#[cfg(debug_assertions)]
pub(super) fn apply_dev_perf_overlay(
    view: &mut AppView,
    window: &mut Window,
    cx: &mut Context<AppView>,
    mut root: Stateful<Div>,
    render_start: std::time::Instant,
) -> Stateful<Div> {
    let (on_dev_move, on_dev_up) = {
        let on_dev_move = Arc::new(cx.listener(
            |this, event: &gpui::MouseMoveEvent, window, cx| {
                if !event.dragging() {
                    if this.dev_perf_monitor.is_dragging {
                        this.dev_perf_monitor.end_drag();
                        cx.notify();
                    }
                    return;
                }
                let vp = window.viewport_size();
                this.dev_perf_monitor
                    .update_drag(event.position, vp.width, vp.height);
                cx.notify();
            },
        ));
        let on_dev_up = Arc::new(cx.listener(
            |this, _event: &gpui::MouseUpEvent, _window, cx| {
                this.dev_perf_monitor.end_drag();
                cx.notify();
            },
        ));
        (on_dev_move, on_dev_up)
    };

    if view.dev_perf_monitor.is_dragging {
        let move_cb = on_dev_move.clone();
        let up_cb = on_dev_up.clone();
        let up_cb_out = on_dev_up.clone();
        let up_cb_right = on_dev_up.clone();
        let up_cb_right_out = on_dev_up.clone();
        let down_cb = on_dev_up.clone();
        let down_cb_right = on_dev_up.clone();

        root = root
            .on_mouse_move({
                let move_cb = on_dev_move.clone();
                move |event, window, cx| {
                    move_cb(event, window, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let up_cb = on_dev_up.clone();
                move |event, window, cx| {
                    up_cb(event, window, cx);
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let up_cb = on_dev_up.clone();
                move |event, window, cx| {
                    up_cb(event, window, cx);
                }
            })
            .child(
                div()
                    .id("dev_perf_drag_capture")
                    .absolute()
                    .inset_0()
                    .cursor_move()
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        cx.stop_propagation();
                        down_cb(&gpui::MouseUpEvent::default(), window, cx);
                    })
                    .on_mouse_down(MouseButton::Right, move |_event, window, cx| {
                        cx.stop_propagation();
                        down_cb_right(&gpui::MouseUpEvent::default(), window, cx);
                    })
                    .on_mouse_move(move |event, window, cx| {
                        move_cb(event, window, cx);
                    })
                    .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                        up_cb(event, window, cx);
                    })
                    .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                        up_cb_out(event, window, cx);
                    })
                    .on_mouse_up(MouseButton::Right, move |event, window, cx| {
                        up_cb_right(event, window, cx);
                    })
                    .on_mouse_up_out(MouseButton::Right, move |event, window, cx| {
                        up_cb_right_out(event, window, cx);
                    }),
            );
    }

    if view.dev_perf_monitor.enabled {
        let current_route = view.current_route;
        let on_toggle_min = cx.listener(|this, _event: &(), _window, cx| {
            this.dev_perf_monitor.minimized = !this.dev_perf_monitor.minimized;
            cx.notify();
        });
        let on_freeze_tel = cx.listener(|this, _event: &(), _window, cx| {
            this.dev_perf_monitor.freeze_telemetry = !this.dev_perf_monitor.freeze_telemetry;
            cx.notify();
        });
        let on_chart_anim = cx.listener(|this, _event: &(), _window, cx| {
            this.dev_perf_monitor.disable_chart_animation =
                !this.dev_perf_monitor.disable_chart_animation;
            cx.notify();
        });
        let on_start_drag = cx.listener(
            |this,
             &(mouse_pos, current_widget_pos): &(Point<Pixels>, Point<Pixels>),
             _window,
             cx| {
                this.dev_perf_monitor
                    .start_drag(mouse_pos, current_widget_pos);
                cx.notify();
            },
        );
        let on_close_hud = cx.listener(|this, _event: &(), _window, cx| {
            this.dev_perf_monitor.enabled = false;
            cx.notify();
        });

        let on_continuous = cx.listener(|this, _event: &(), _window, cx| {
            this.dev_perf_monitor.continuous_mode = !this.dev_perf_monitor.continuous_mode;
            cx.notify();
        });

        let on_hover_perf_control = cx.listener(
            |this, &(ctrl, is_hovered): &(&'static str, bool), _window, cx| {
                let new_ctrl = if is_hovered {
                    Some(ctrl)
                } else if this.dev_perf_monitor.hovered_control == Some(ctrl) {
                    None
                } else {
                    return;
                };
                if this.dev_perf_monitor.hovered_control != new_ctrl {
                    this.dev_perf_monitor.set_hovered_control(new_ctrl);
                    cx.notify();
                }
            },
        );

        let on_dev_move_widget = on_dev_move.clone();
        let on_dev_up_widget = on_dev_up.clone();

        let perf_widget = crate::widgets::dev_perf_monitor::DevPerfMonitor::new(
            view.dev_perf_monitor.snapshot(),
            current_route,
            move |window, cx| {
                on_toggle_min(&(), window, cx);
            },
            move |window, cx| {
                on_freeze_tel(&(), window, cx);
            },
            move |window, cx| {
                on_chart_anim(&(), window, cx);
            },
            move |window, cx| {
                on_continuous(&(), window, cx);
            },
            move |mouse_pos, current_pos, window, cx| {
                on_start_drag(&(mouse_pos, current_pos), window, cx);
            },
            move |mouse_pos, is_pressed, window, cx| {
                let event = gpui::MouseMoveEvent {
                    position: mouse_pos,
                    pressed_button: if is_pressed {
                        Some(MouseButton::Left)
                    } else {
                        None
                    },
                    modifiers: gpui::Modifiers::default(),
                };
                on_dev_move_widget(&event, window, cx);
            },
            move |window, cx| {
                let event = gpui::MouseUpEvent {
                    button: MouseButton::Left,
                    position: gpui::point(px(0.0), px(0.0)),
                    modifiers: gpui::Modifiers::default(),
                    click_count: 1,
                };
                on_dev_up_widget(&event, window, cx);
            },
            move |window, cx| {
                on_close_hud(&(), window, cx);
            },
        )
        .on_hover_control(move |ctrl, is_hovered, window, cx| {
            on_hover_perf_control(&(ctrl, is_hovered), window, cx);
        });

        root = root.child(perf_widget);
    }

    #[allow(clippy::cast_precision_loss)]
    let draw_ms = render_start.elapsed().as_secs_f32() * 1000.0;
    view.dev_perf_monitor.record_frame(draw_ms);
    if view.dev_perf_monitor.continuous_mode && view.dev_perf_monitor.enabled {
        window.request_animation_frame();
    }

    root
}

#[cfg(not(debug_assertions))]
pub(super) fn apply_dev_perf_overlay(
    _view: &mut AppView,
    _window: &mut Window,
    _cx: &mut Context<AppView>,
    root: Div,
    _render_start: std::time::Instant,
) -> Stateful<Div> {
    root
}