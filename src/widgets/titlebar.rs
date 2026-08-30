use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, WindowControlArea, div, ease_in_out, px, svg,
};

use crate::features::navigation::AppRoute;
use crate::shared::theme::Theme;
use crate::shared::ui::{BreadcrumbItem, Breadcrumbs, TooltipState};
use crate::widgets::sidebar::{TooltipHoverHandler, lerp_item_bg, lerp_item_text};
use crate::widgets::window_controls::WindowControls;

pub type SidebarToggleHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type HoverToggleHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type HoverControlHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type HoverBreadcrumbHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;
pub type TitlebarNavigateHandler =
    Arc<dyn Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static>;
pub type WindowCloseHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Titlebar {
    current_route: AppRoute,
    sidebar_expanded: bool,
    sidebar_toggle_hovered: bool,
    hovered_win_control: Option<&'static str>,
    hovered_breadcrumb: Option<&'static str>,
    on_toggle_sidebar: Option<SidebarToggleHandler>,
    on_hover_sidebar_toggle: Option<HoverToggleHandler>,
    on_hover_win_control: Option<HoverControlHandler>,
    on_hover_breadcrumb: Option<HoverBreadcrumbHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
    on_navigate: Option<TitlebarNavigateHandler>,
    on_close_window: Option<WindowCloseHandler>,
}

impl Default for Titlebar {
    fn default() -> Self {
        Self::new(AppRoute::Dashboard, false, false, None)
    }
}

impl Titlebar {
    #[must_use]
    pub fn new(
        current_route: AppRoute,
        sidebar_expanded: bool,
        sidebar_toggle_hovered: bool,
        hovered_win_control: Option<&'static str>,
    ) -> Self {
        Self {
            current_route,
            sidebar_expanded,
            sidebar_toggle_hovered,
            hovered_win_control,
            hovered_breadcrumb: None,
            on_toggle_sidebar: None,
            on_hover_sidebar_toggle: None,
            on_hover_win_control: None,
            on_hover_breadcrumb: None,
            on_hover_tooltip: None,
            on_navigate: None,
            on_close_window: None,
        }
    }

    #[must_use]
    pub fn on_toggle_sidebar(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_sidebar = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_sidebar_toggle(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_sidebar_toggle = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_win_control(
        mut self,
        handler: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_win_control = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn hovered_breadcrumb(mut self, hovered: Option<&'static str>) -> Self {
        self.hovered_breadcrumb = hovered;
        self
    }

    #[must_use]
    pub fn on_hover_breadcrumb(
        mut self,
        handler: impl Fn(&'static str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_breadcrumb = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_tooltip(
        mut self,
        handler: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_navigate(
        mut self,
        handler: impl Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_navigate = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close_window(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close_window = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Titlebar {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let on_toggle_sidebar = self.on_toggle_sidebar.clone();
        let on_hover_toggle = self.on_hover_sidebar_toggle;
        let on_hover_ctrl = self.on_hover_win_control;
        let on_hover_breadcrumb = self.on_hover_breadcrumb;
        let on_hover_tooltip = self.on_hover_tooltip;
        let on_close_window = self.on_close_window;
        let current_route = self.current_route;
        let sidebar_expanded = self.sidebar_expanded;
        let is_toggle_hovered = self.sidebar_toggle_hovered;
        let hovered_win_control = self.hovered_win_control;
        let hovered_breadcrumb = self.hovered_breadcrumb;

        // Smooth spring hover on sidebar toggle button (0.0 -> 0.5)
        let toggle_hover_target: f32 = if is_toggle_hovered { 0.5 } else { 0.0 };
        let toggle_hover_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(toggle_hover_target)
            .with_epsilon(0.005);

        let accent_cyan = theme.accent_cyan;
        let toggle_text_color = lerp_item_text(&theme, toggle_hover_target);

        // Sidebar panel icon:
        // - sidebar collapsed (false) -> panel-left-open.svg (arrow -> pointing right to expand)
        // - sidebar expanded (true) -> panel-left-close.svg (arrow <- pointing left to collapse)
        let (icon_path, anim_id) = if sidebar_expanded {
            ("icons/panel-left-close.svg", "panel_morph_close")
        } else {
            ("icons/panel-left-open.svg", "panel_morph_open")
        };

        let sidebar_icon = svg()
            .path(icon_path)
            .size(px(16.0))
            .text_color(toggle_text_color)
            .with_animation(
                ElementId::Name(anim_id.into()),
                Animation::new(Duration::from_millis(180)).with_easing(ease_in_out),
                gpui::Styled::opacity,
            );

        let mut win_controls = WindowControls::new(hovered_win_control);
        if let Some(h) = on_hover_ctrl {
            win_controls = win_controls.on_hover_control(move |ctrl, hov, window, cx| {
                h(ctrl, hov, window, cx);
            });
        }
        if let Some(ref th) = on_hover_tooltip {
            let th_copy = th.clone();
            win_controls = win_controls.on_hover_tooltip(move |tt, window, cx| {
                th_copy(tt, window, cx);
            });
        }
        if let Some(ref close_h) = on_close_window {
            let close_copy = close_h.clone();
            win_controls = win_controls.on_close(move |window, cx| {
                close_copy(window, cx);
            });
        }

        // Top-level breadcrumb text with hierarchy for sub-routes
        let breadcrumbs = match current_route {
            AppRoute::CpuDetail
            | AppRoute::RamDetail
            | AppRoute::DiskDetail(_)
            | AppRoute::NetworkDetail(_)
            | AppRoute::GpuDetail(_) => {
                let on_nav_dash = self.on_navigate.clone();
                let mut dashboard_item =
                    BreadcrumbItem::new("dash", rust_i18n::t!("nav.dashboard"))
                        .hovered(hovered_breadcrumb == Some("dash"))
                        .on_click(move |window, cx| {
                            if let Some(ref h) = on_nav_dash {
                                h(AppRoute::Dashboard, window, cx);
                            }
                        });
                if let Some(handler) = on_hover_breadcrumb {
                    dashboard_item = dashboard_item.on_hover(move |hovered, window, cx| {
                        handler("dash", hovered, window, cx);
                    });
                }

                Breadcrumbs::new(format!("titlebar_{}_breadcrumbs", current_route.id()))
                    .item(dashboard_item)
                    .item(
                        BreadcrumbItem::new(current_route.id(), current_route.title())
                            .current(true),
                    )
            }
            _ => Breadcrumbs::new(current_route.id())
                .item(BreadcrumbItem::new("current", current_route.title()).current(true)),
        };

        let toggle_tt_hov = on_hover_tooltip.clone();
        let toggle_tt_move = on_hover_tooltip;

        // Symmetrical layout ensuring center is at the EXACT geometric center of the window:
        // Left width = 104px, Right width = 104px (3 buttons * 32px + 2 gaps * 4px = 104px)
        let sidebar_toggle_btn = div()
            .id("sidebar_toggle_btn")
            .flex()
            .items_center()
            .justify_center()
            .size(px(32.0))
            .rounded(px(6.0))
            .cursor_pointer()
            .active(move |s| s.bg(theme.accent_active_bg))
            .on_hover(move |&hov, window, cx| {
                if let Some(ref h) = on_hover_toggle {
                    h(&hov, window, cx);
                }
                if let Some(ref th) = toggle_tt_hov {
                    if hov {
                        let text = if sidebar_expanded {
                            rust_i18n::t!("titlebar.toggle_sidebar_collapse").to_string()
                        } else {
                            rust_i18n::t!("titlebar.toggle_sidebar_expand").to_string()
                        };
                        let pos = window.mouse_position();
                        th(
                            Some(TooltipState {
                                text: text.into(),
                                cursor_pos: pos,
                            }),
                            window,
                            cx,
                        );
                    } else {
                        th(None, window, cx);
                    }
                }
            })
            .on_mouse_move(move |event, window, cx| {
                if let Some(ref th) = toggle_tt_move {
                    let text = if sidebar_expanded {
                        rust_i18n::t!("titlebar.toggle_sidebar_collapse").to_string()
                    } else {
                        rust_i18n::t!("titlebar.toggle_sidebar_expand").to_string()
                    };
                    th(
                        Some(TooltipState {
                            text: text.into(),
                            cursor_pos: event.position,
                        }),
                        window,
                        cx,
                    );
                }
            })
            .on_click(move |event, window, cx| {
                if let Some(ref handler) = on_toggle_sidebar {
                    handler(event, window, cx);
                }
            })
            .with_spring(
                "sidebar_toggle_bg_spring",
                toggle_hover_spring,
                move |btn, val| {
                    let bg = lerp_item_bg(accent_cyan, val);
                    btn.bg(bg)
                },
            )
            .child(sidebar_icon);

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(40.0))
            .px(px(4.0))
            .py(px(4.0))
            .bg(theme.titlebar_bg)
            .w_full()
            // Left container (104px wide matching right window controls width)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_start()
                    .w(px(104.0))
                    .h_full()
                    .child(sidebar_toggle_btn)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag)
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move();
                            }),
                    ),
            )
            // Center container (Takes all remaining width, centering breadcrumb precisely in the window midpoint)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .h_full()
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag)
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move();
                            }),
                    )
                    .child(breadcrumbs)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag)
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move();
                            }),
                    ),
            )
            // Right container (104px wide containing window controls)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .w(px(104.0))
                    .h_full()
                    .child(win_controls),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use gpui::{
        Context, IntoElement, Modifiers, Render, TestAppContext, VisualTestContext, px, size,
    };

    use super::{AppRoute, Titlebar};

    struct TestTitlebar {
        navigated: Arc<AtomicBool>,
    }

    impl Render for TestTitlebar {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let navigated = Arc::clone(&self.navigated);
            Titlebar::new(AppRoute::CpuDetail, false, false, None).on_navigate(
                move |route, _window, _cx| {
                    if route == AppRoute::Dashboard {
                        navigated.store(true, Ordering::Relaxed);
                    }
                },
            )
        }
    }

    #[gpui::test]
    fn parent_breadcrumb_is_clickable(cx: &mut TestAppContext) {
        let navigated = Arc::new(AtomicBool::new(false));
        let window = cx.open_window(size(px(600.0), px(200.0)), {
            let navigated = Arc::clone(&navigated);
            move |_, _| TestTitlebar { navigated }
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let bounds = cx
            .debug_bounds("breadcrumb_dash")
            .expect("parent breadcrumb must be rendered");

        cx.simulate_click(bounds.center(), Modifiers::none());

        assert!(navigated.load(Ordering::Relaxed));
    }
}
