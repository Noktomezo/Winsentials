use std::sync::Arc;

use gpui::{
    AnimationExt, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, SpringAnimation,
    SpringConfig, StatefulInteractiveElement, Styled, Window, WindowControlArea, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::{TooltipHoverHandler, lerp_item_bg, lerp_item_text};

pub type ControlHoverHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type ControlClickHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct WindowControls {
    hovered_control: Option<&'static str>,
    on_hover_control: Option<ControlHoverHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
    on_close: Option<ControlClickHandler>,
}

impl Default for WindowControls {
    fn default() -> Self {
        Self::new(None)
    }
}

impl WindowControls {
    #[must_use]
    pub fn new(hovered_control: Option<&'static str>) -> Self {
        Self {
            hovered_control,
            on_hover_control: None,
            on_hover_tooltip: None,
            on_close: None,
        }
    }

    #[must_use]
    pub fn on_hover_control(
        mut self,
        handler: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_control = Some(Arc::new(handler));
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
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for WindowControls {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let hovered_control = self.hovered_control;
        let on_hover = self.on_hover_control;
        let on_hover_min = on_hover.clone();
        let on_hover_close = on_hover;

        let tooltip_handler = self.on_hover_tooltip;
        let on_close_btn = self.on_close;
        let tt_min_hov = tooltip_handler.clone();
        let tt_min_move = tooltip_handler.clone();
        let tt_close_hov = tooltip_handler.clone();
        let tt_close_move = tooltip_handler;

        let is_min_hovered = hovered_control == Some("min");
        let is_close_hovered = hovered_control == Some("close");

        let min_target: f32 = if is_min_hovered { 0.5 } else { 0.0 };
        let close_target: f32 = if is_close_hovered { 0.5 } else { 0.0 };

        let min_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(min_target)
            .with_epsilon(0.005);
        let close_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(close_target)
            .with_epsilon(0.005);

        let accent_blue = theme.accent_blue;
        let theme_ref = theme;
        let red_hover_bg = theme.accent_red;

        let min_text_color = lerp_item_text(&theme_ref, min_target);
        let close_text_color = if close_target > 0.0 {
            theme_ref.accent_red
        } else {
            theme_ref.text_primary
        };

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                div()
                    .id("win_minimize")
                    .window_control_area(WindowControlArea::Min)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(32.0))
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .active(move |s| s.bg(theme.accent_active_bg))
                    .on_hover(move |&hov, window, cx| {
                        if let Some(ref h) = on_hover_min {
                            h("min", &hov, window, cx);
                        }
                        if let Some(ref th) = tt_min_hov {
                            if hov {
                                let pos = window.mouse_position();
                                th(
                                    Some(TooltipState {
                                        text: rust_i18n::t!("titlebar.minimize").to_string().into(),
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
                        if let Some(ref th) = tt_min_move {
                            th(
                                Some(TooltipState {
                                    text: rust_i18n::t!("titlebar.minimize").to_string().into(),
                                    cursor_pos: event.position,
                                }),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_click(|_, _, cx| {
                        cx.hide();
                    })
                    .with_spring("win_min_spring", min_spring, move |btn, val| {
                        let bg = lerp_item_bg(accent_blue, val);
                        btn.bg(bg)
                    })
                    .child(
                        Icon::new("icons/minus.svg")
                            .size(px(14.0))
                            .color(min_text_color),
                    ),
            )
            .child(
                div()
                    .id("win_close")
                    .window_control_area(WindowControlArea::Close)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(32.0))
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .active(move |s| s.bg(theme.accent_red))
                    .on_hover(move |&hov, window, cx| {
                        if let Some(ref h) = on_hover_close {
                            h("close", &hov, window, cx);
                        }
                        if let Some(ref th) = tt_close_hov {
                            if hov {
                                let pos = window.mouse_position();
                                th(
                                    Some(TooltipState {
                                        text: rust_i18n::t!("titlebar.close").to_string().into(),
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
                        if let Some(ref th) = tt_close_move {
                            th(
                                Some(TooltipState {
                                    text: rust_i18n::t!("titlebar.close").to_string().into(),
                                    cursor_pos: event.position,
                                }),
                                window,
                                cx,
                            );
                        }
                    })
                    .on_click(move |_, window, cx| {
                        if let Some(ref h) = on_close_btn {
                            h(window, cx);
                        } else {
                            cx.quit();
                        }
                    })
                    .with_spring("win_close_spring", close_spring, move |btn, val| {
                        let bg = lerp_item_bg(red_hover_bg, val);
                        btn.bg(bg)
                    })
                    .child(
                        Icon::new("icons/x.svg")
                            .size(px(14.0))
                            .color(close_text_color),
                    ),
            )
    }
}
