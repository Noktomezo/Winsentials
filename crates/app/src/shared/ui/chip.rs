use std::sync::Arc;

use gpui::{
    AnimationExt, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::motion::lerp_item_bg;
use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type ClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type MouseDownHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type HoverHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    selected: bool,
    destructive: bool,
    disabled: bool,
    spring: Option<(SpringAnimation<f32>, Rgba)>,
    on_click: Option<ClickHandler>,
    on_mouse_down: Option<MouseDownHandler>,
    on_hover: Option<HoverHandler>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            selected: false,
            destructive: false,
            disabled: false,
            spring: None,
            on_click: None,
            on_mouse_down: None,
            on_hover: None,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_mouse_down(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_mouse_down = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn spring(mut self, spring: SpringAnimation<f32>, accent: Rgba) -> Self {
        self.spring = Some((spring, accent));
        self
    }

    #[must_use]
    pub fn on_hover(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Chip {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);

        let (bg, hover_bg, active_bg, text_color, border_color) = if self.destructive {
            (
                theme.accent_red.opacity(0.18),
                theme.accent_red.opacity(0.28),
                theme.accent_red.opacity(0.40),
                theme.accent_red,
                Some(theme.accent_red.opacity(0.35)),
            )
        } else if self.selected {
            (
                theme.button_selected,
                theme.accent_hover_bg,
                theme.accent_active_bg,
                theme.text_primary,
                Some(theme.card_border),
            )
        } else {
            (
                theme.input_bg,
                theme.button_hover,
                theme.accent_active_bg,
                theme.text_muted,
                Some(theme.card_border),
            )
        };

        let spring_id = format!("{:?}_bg_spring", self.id);

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(5.0))
            .px(px(10.0))
            .py(px(4.0))
            .rounded_md()
            .bg(bg)
            .text_xs()
            .font_weight(if self.selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(text_color);

        if let Some(border) = border_color {
            base = base.border_1().border_color(border);
        }

        if self.disabled {
            base = base.opacity(0.45);
        } else if self.spring.is_some() {
            base = base.cursor_pointer().active(move |s| s.bg(active_bg));

            if let Some(on_hover) = self.on_hover {
                base = base.on_hover(move |&hov, window, cx| {
                    (on_hover)(hov, window, cx);
                });
            }

            if let Some(on_click) = self.on_click {
                base = base.on_click(move |event, window, cx| {
                    (on_click)(event, window, cx);
                });
            }

            if let Some(on_mouse_down) = self.on_mouse_down {
                base = base.on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    cx.stop_propagation();
                    (on_mouse_down)(window, cx);
                });
            }
        } else {
            base = base
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(active_bg));

            if let Some(on_click) = self.on_click {
                base = base.on_click(move |event, window, cx| {
                    (on_click)(event, window, cx);
                });
            }

            if let Some(on_mouse_down) = self.on_mouse_down {
                base = base.on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    cx.stop_propagation();
                    (on_mouse_down)(window, cx);
                });
            }
        }

        let icon_el = self
            .icon
            .map(|p| Icon::new(p).size(px(13.0)).color(text_color));

        let content = base.children(icon_el).child(self.label);

        if let Some((spring, accent)) = self.spring {
            content
                .with_spring(
                    ElementId::Name(spring_id.into()),
                    spring,
                    move |btn, val| {
                        let bg = lerp_item_bg(accent, val);
                        btn.bg(bg)
                    },
                )
                .into_any_element()
        } else {
            content.into_any_element()
        }
    }
}
