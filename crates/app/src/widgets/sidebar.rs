use std::sync::Arc;

use gpui::{
    AnimationExt, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Window, div,
    px,
};

use crate::features::navigation::AppRoute;
use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;
use crate::shared::ui::icon::Icon;

pub type RouteNavigateHandler = Arc<dyn Fn(&AppRoute, &mut Window, &mut App) + 'static>;
pub type RouteHoverHandler = Arc<dyn Fn(&(AppRoute, bool), &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Sidebar {
    expanded: bool,
    current_route: AppRoute,
    hovered_route: Option<AppRoute>,
    on_navigate: Option<RouteNavigateHandler>,
    on_hover_route: Option<RouteHoverHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new(false, AppRoute::Dashboard, None)
    }
}

impl Sidebar {
    #[must_use]
    pub fn new(expanded: bool, current_route: AppRoute, hovered_route: Option<AppRoute>) -> Self {
        Self {
            expanded,
            current_route,
            hovered_route,
            on_navigate: None,
            on_hover_route: None,
            on_hover_tooltip: None,
        }
    }

    #[must_use]
    pub fn on_navigate(
        mut self,
        handler: impl Fn(&AppRoute, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_route(
        mut self,
        handler: impl Fn(&(AppRoute, bool), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_route = Some(Arc::new(handler));
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
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let on_navigate = self.on_navigate.clone();
        let on_hover_route = self.on_hover_route.clone();
        let on_hover_tooltip = self.on_hover_tooltip.clone();
        let expanded = self.expanded;
        let current_route = self.current_route;
        let hovered_route = self.hovered_route;

        let mut top_stack = div().flex().flex_col().items_center().gap(px(4.0)).w_full();

        for route in AppRoute::TOP_NAV {
            let is_selected = current_route == route;
            let is_hovered = hovered_route == Some(route);
            let target_state = if is_selected {
                1.0
            } else if is_hovered {
                0.5
            } else {
                0.0
            };

            let nav_handler = on_navigate.clone();
            let hover_handler = on_hover_route.clone();
            let tooltip_handler = on_hover_tooltip.clone();

            top_stack = top_stack.child(render_sidebar_item(
                route.id(),
                route.icon(),
                route.title().into(),
                target_state,
                expanded,
                &theme,
                move |window, cx| {
                    if let Some(ref h) = nav_handler {
                        h(&route, window, cx);
                    }
                },
                move |hovered, window, cx| {
                    if let Some(ref h) = hover_handler {
                        h(&(route, hovered), window, cx);
                    }
                },
                tooltip_handler,
            ));
        }

        let mut bottom_stack = div().flex().flex_col().items_center().gap(px(4.0)).w_full();

        for route in AppRoute::BOTTOM_NAV {
            let is_selected = current_route == route;
            let is_hovered = hovered_route == Some(route);
            let target_state = if is_selected {
                1.0
            } else if is_hovered {
                0.5
            } else {
                0.0
            };

            let nav_handler = on_navigate.clone();
            let hover_handler = on_hover_route.clone();
            let tooltip_handler = on_hover_tooltip.clone();

            bottom_stack = bottom_stack.child(render_sidebar_item(
                route.id(),
                route.icon(),
                route.title().into(),
                target_state,
                expanded,
                &theme,
                move |window, cx| {
                    if let Some(ref h) = nav_handler {
                        h(&route, window, cx);
                    }
                },
                move |hovered, window, cx| {
                    if let Some(ref h) = hover_handler {
                        h(&(route, hovered), window, cx);
                    }
                },
                tooltip_handler,
            ));
        }

        let target_width = if expanded { px(200.0) } else { px(40.0) };
        let spring = SpringAnimation::new(SpringConfig::new(320.0, 26.0, 1.0))
            .to(target_width)
            .with_epsilon(0.5);

        div()
            .id("sidebar_root")
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .h_full()
            .p(px(4.0))
            .bg(theme.sidebar_bg)
            .overflow_hidden()
            .with_spring("sidebar_spring", spring, move |sidebar, width| {
                sidebar.w(width)
            })
            .child(top_stack)
            .child(bottom_stack)
    }
}

pub use crate::shared::motion::{lerp_item_bg, lerp_item_text, lerp_rgba};

#[allow(clippy::too_many_arguments)]
fn render_sidebar_item(
    id: &'static str,
    icon: &'static str,
    label: SharedString,
    target_state: f32,
    expanded: bool,
    theme: &Theme,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
    on_hover: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_hover_tooltip: Option<TooltipHoverHandler>,
) -> impl IntoElement {
    let accent_blue = theme.accent_blue;

    let state_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
        .to(target_state)
        .with_epsilon(0.005);

    let current_color = lerp_item_text(theme, target_state);

    // Fixed 32px icon container (always fits 40px bar)
    let icon_container = div()
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .child(Icon::new(icon).size(px(16.0)).color(current_color));

    // Smooth spring-driven text container with fade and slide
    let target_progress = if expanded { 1.0 } else { 0.0 };
    let progress_spring = SpringAnimation::new(SpringConfig::new(320.0, 26.0, 1.0))
        .to(target_progress)
        .with_epsilon(0.01);

    let text_container = div()
        .flex_1()
        .overflow_hidden()
        .truncate()
        .pr(px(6.0))
        .with_spring(
            ElementId::Name(format!("{id}_progress").into()),
            progress_spring,
            move |text_box, progress| {
                let slide = (1.0 - progress) * -16.0;
                text_box.opacity(progress).ml(px(slide))
            },
        )
        .child(
            div()
                .text_size(px(13.0))
                .text_color(current_color)
                .font_weight(if target_state >= 0.5 {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::NORMAL
                })
                .truncate()
                .child(label.clone()),
        );

    let label_tooltip_hov = label.clone();
    let label_tooltip_move = label;
    let tooltip_h_hov = on_hover_tooltip.clone();
    let tooltip_h_move = on_hover_tooltip;

    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(32.0))
        .w_full()
        .rounded(px(6.0))
        .cursor_pointer()
        .overflow_hidden()
        .on_hover(move |&hovered, window, cx| {
            on_hover(hovered, window, cx);
            if let Some(ref th) = tooltip_h_hov {
                if hovered && !expanded {
                    let pos = window.mouse_position();
                    th(
                        Some(TooltipState {
                            text: label_tooltip_hov.clone(),
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
            if !expanded {
                if let Some(ref th) = tooltip_h_move {
                    th(
                        Some(TooltipState {
                            text: label_tooltip_move.clone(),
                            cursor_pos: event.position,
                        }),
                        window,
                        cx,
                    );
                }
            }
        })
        .on_click(move |_, window, cx| {
            on_click(window, cx);
        })
        .with_spring(
            ElementId::Name(format!("{id}_state_spring").into()),
            state_spring,
            move |btn, val| {
                let bg = lerp_item_bg(accent_blue, val);
                btn.bg(bg)
            },
        )
        .child(icon_container)
        .child(text_container)
}
