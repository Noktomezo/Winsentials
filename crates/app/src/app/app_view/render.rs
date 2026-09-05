use gpui::{
    Context, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseUpEvent, NavigationDirection, ParentElement, Render, SharedString, Styled, Window,
    div, px,
};

use crate::features::navigation::AppRoute;
use crate::shared::theme::Theme;
use crate::shared::ui::{Tooltip, TooltipState};
use crate::widgets::sidebar::Sidebar;
use crate::widgets::titlebar::Titlebar;

use super::AppView;
#[cfg(debug_assertions)]
use super::render_hud::apply_dev_perf_overlay;

impl Render for AppView {
    #[allow(clippy::too_many_lines, unused_variables)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(debug_assertions)]
        let render_start = std::time::Instant::now();

        let theme = Theme::get(cx);
        let sidebar_expanded = self.sidebar_expanded;
        let sidebar_toggle_hovered = self.sidebar_toggle_hovered;
        let hovered_win_control = self.hovered_win_control;
        let hovered_titlebar_breadcrumb = self.hovered_titlebar_breadcrumb;
        let current_route = self.current_route;
        let hovered_route = self.hovered_route;
        let active_tooltip = self.active_tooltip.clone();

        let on_hover_sidebar_toggle = cx.listener(|this, &hovered: &bool, _window, cx| {
            this.sidebar_toggle_hovered = hovered;
            cx.notify();
        });

        let on_toggle_sidebar = cx.listener(|this, _event: &(), window, cx| {
            this.toggle_sidebar(window, cx);
        });

        let on_navigate = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_hover_route = cx.listener(
            |this, &(route, is_hovered): &(AppRoute, bool), window, cx| {
                this.set_hovered_route(route, is_hovered, window, cx);
            },
        );

        let on_hover_win_control = cx.listener(
            |this, &(ctrl, is_hovered): &(&'static str, bool), _window, cx| {
                if is_hovered {
                    if this.hovered_win_control != Some(ctrl) {
                        this.hovered_win_control = Some(ctrl);
                        cx.notify();
                    }
                } else if this.hovered_win_control == Some(ctrl) {
                    this.hovered_win_control = None;
                    cx.notify();
                }
            },
        );

        let on_hover_titlebar_breadcrumb = cx.listener(
            |this, &(id, is_hovered): &(&'static str, bool), _window, cx| {
                this.set_hovered_titlebar_breadcrumb(id, is_hovered, cx);
            },
        );

        let titlebar_tooltip_listener =
            cx.listener(|this, tooltip: &Option<TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let sidebar_tooltip_listener =
            cx.listener(|this, tooltip: &Option<TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let on_navigate_titlebar = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_close_win = cx.listener(|this, _event: &(), window, cx| {
            this.handle_window_close(window, cx);
        });

        let titlebar = Titlebar::new(
            current_route,
            sidebar_expanded,
            sidebar_toggle_hovered,
            hovered_win_control,
        )
        .hovered_breadcrumb(hovered_titlebar_breadcrumb)
        .on_hover_breadcrumb(move |id, is_hovered, window, cx| {
            on_hover_titlebar_breadcrumb(&(id, is_hovered), window, cx);
        })
        .on_navigate(move |route, window, cx| {
            on_navigate_titlebar(&route, window, cx);
        })
        .on_hover_sidebar_toggle(move |hovered, window, cx| {
            on_hover_sidebar_toggle(hovered, window, cx);
        })
        .on_toggle_sidebar(move |_event, window, cx| {
            on_toggle_sidebar(&(), window, cx);
        })
        .on_hover_win_control(move |ctrl, is_hovered, window, cx| {
            on_hover_win_control(&(ctrl, *is_hovered), window, cx);
        })
        .on_hover_tooltip(move |tooltip, window, cx| {
            titlebar_tooltip_listener(&tooltip, window, cx);
        })
        .on_close_window(move |window, cx| {
            on_close_win(&(), window, cx);
        });

        let sidebar = Sidebar::new(sidebar_expanded, current_route, hovered_route)
            .on_navigate(move |route, window, cx| {
                on_navigate(route, window, cx);
            })
            .on_hover_route(move |pair, window, cx| {
                on_hover_route(pair, window, cx);
            })
            .on_hover_tooltip(move |tooltip, window, cx| {
                sidebar_tooltip_listener(&tooltip, window, cx);
            });

        let main_panel = self.render_main_panel(cx);

        let content_row = div()
            .flex()
            .flex_row()
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .child(sidebar)
            .child(main_panel);

        let focus_handle = self
            .focus_handle
            .get_or_insert_with(|| cx.focus_handle())
            .clone();

        if window.focused(cx).is_none() {
            focus_handle.focus(window, cx);
        }

        let mut root = div()
            .id("app_root")
            .track_focus(&focus_handle)
            .relative()
            .font_family("IBM Plex Sans")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.window_bg)
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                match event.button {
                    MouseButton::Navigate(NavigationDirection::Back) => {
                        cx.stop_propagation();
                        this.navigate_back(window, cx);
                    }
                    MouseButton::Navigate(NavigationDirection::Forward) => {
                        cx.stop_propagation();
                        this.navigate_forward(window, cx);
                    }
                    _ => {}
                }
            }))
            .capture_any_mouse_up(cx.listener(|_this, event: &MouseUpEvent, _window, cx| {
                if matches!(event.button, MouseButton::Navigate(_)) {
                    cx.stop_propagation();
                }
            }))
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let is_alt = event.keystroke.modifiers.alt;
                if key == "escape" {
                    cx.stop_propagation();
                    this.handle_escape(window, cx);
                } else if (is_alt && (key == "left" || key == "arrowleft")) || key == "back" {
                    cx.stop_propagation();
                    this.navigate_back(window, cx);
                } else if (is_alt && (key == "right" || key == "arrowright")) || key == "forward" {
                    cx.stop_propagation();
                    this.navigate_forward(window, cx);
                } else if is_alt && (key == "up" || key == "arrowup") {
                    cx.stop_propagation();
                    this.navigate_up(window, cx);
                }
            }))
            .child(titlebar)
            .child(content_row);

        if let Some(ref tt) = active_tooltip {
            root = root.child(Tooltip::new(tt.text.clone(), tt.cursor_pos));
        }

        if !self.toasts.is_empty() {
            let on_dismiss_toast = cx.listener(|this, toast_id: &str, _window, cx| {
                this.dismiss_toast(toast_id, cx);
            });
            let on_hover_toast_btn = cx.listener(
                |this, &(ref t_id, idx, is_hov): &(SharedString, usize, bool), _window, cx| {
                    this.set_hovered_toast_button(t_id, idx, is_hov, cx);
                },
            );
            let on_hover_stack = cx.listener(|this, &is_hov: &bool, _window, cx| {
                this.set_toast_stack_expanded(is_hov, cx);
            });

            let stack_el = crate::shared::ui::ToastStack::new(self.toasts.clone())
                .closing_id(self.closing_toast_id.clone())
                .hovered_toast_button(self.hovered_toast_button.clone())
                .expanded(self.toast_stack_expanded)
                .on_dismiss(move |toast_id, window, cx| {
                    on_dismiss_toast(toast_id, window, cx);
                })
                .on_hover_button(move |toast_id, idx, is_hov, window, cx| {
                    on_hover_toast_btn(&(toast_id.to_string().into(), idx, *is_hov), window, cx);
                })
                .on_hover_stack(move |is_hov, window, cx| {
                    on_hover_stack(is_hov, window, cx);
                })
                .into_any_element();

            root = root.child(gpui::deferred(stack_el).with_priority(200));
        }

        #[cfg(debug_assertions)]
        let root = apply_dev_perf_overlay(self, window, cx, root, render_start);

        root
    }
}