use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, ease_in_out, px,
};

use crate::components::button::{Button, ButtonVariant};
use crate::components::icon::Icon;
use crate::theme::Theme;

pub type ModalActionHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModalVariant {
    #[default]
    Warning,
    Destructive,
    Info,
}

#[derive(IntoElement)]
pub struct Modal {
    id: ElementId,
    title: SharedString,
    description: SharedString,
    confirm_label: SharedString,
    cancel_label: SharedString,
    variant: ModalVariant,
    custom_content: Option<AnyElement>,
    on_confirm: Option<ModalActionHandler>,
    on_cancel: Option<ModalActionHandler>,
    on_close: Option<ModalActionHandler>,
}

impl Modal {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: SharedString::default(),
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            variant: ModalVariant::Warning,
            custom_content: None,
            on_confirm: None,
            on_cancel: None,
            on_close: None,
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = description.into();
        self
    }

    #[must_use]
    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    #[must_use]
    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    #[must_use]
    pub const fn variant(mut self, variant: ModalVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn custom_content(mut self, content: impl IntoElement) -> Self {
        self.custom_content = Some(content.into_any_element());
        self
    }

    #[must_use]
    pub fn on_confirm(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_confirm = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let close_action = self.on_close.clone().or_else(|| self.on_cancel.clone());
        let on_close_backdrop = close_action.clone();
        let on_close_x = close_action;
        let on_cancel_btn = self.on_cancel.clone();
        let on_confirm_btn = self.on_confirm;

        let (icon_path, icon_color) = match self.variant {
            ModalVariant::Warning => ("icons/triangle-alert.svg", theme.accent_orange),
            ModalVariant::Destructive => ("icons/triangle-alert.svg", theme.accent_red),
            ModalVariant::Info => ("icons/info.svg", theme.accent_blue),
        };

        let confirm_btn_variant = match self.variant {
            ModalVariant::Destructive => ButtonVariant::Destructive,
            ModalVariant::Warning | ModalVariant::Info => ButtonVariant::Primary,
        };

        let icon_box = div()
            .size(px(36.0))
            .rounded(px(8.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.input_border)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .child(Icon::new(icon_path).size(px(18.0)).color(icon_color));

        let close_btn = {
            div()
                .id("modal_close_btn")
                .size(px(24.0))
                .rounded(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(theme.text_muted)
                .hover(|s| s.text_color(theme.text_primary).bg(theme.input_bg))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    if let Some(ref cb) = on_close_x {
                        cb(window, cx);
                    }
                })
                .child(Icon::new("icons/x.svg").size(px(14.0)))
        };

        let header = div()
            .flex()
            .items_start()
            .justify_between()
            .w_full()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .flex_1()
                    .min_w(px(0.0))
                    .child(icon_box)
                    .child(
                        div()
                            .text_size(px(15.0))
                            .line_height(px(20.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child(self.title),
                    ),
            )
            .child(close_btn);

        let body = div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .w_full()
            .when(!self.description.is_empty(), |this| {
                this.child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .text_color(theme.text_muted)
                        .w_full()
                        .child(self.description),
                )
            })
            .children(self.custom_content);

        let cancel_action = on_cancel_btn;
        let confirm_action = on_confirm_btn;

        let footer = div()
            .flex()
            .items_center()
            .justify_end()
            .gap(px(10.0))
            .w_full()
            .mt(px(4.0))
            .child(
                Button::new("modal_cancel_action", self.cancel_label)
                    .variant(ButtonVariant::Secondary)
                    .on_click(move |_event, window, cx| {
                        if let Some(ref cb) = cancel_action {
                            cb(window, cx);
                        }
                    }),
            )
            .child(
                Button::new("modal_confirm_action", self.confirm_label)
                    .variant(confirm_btn_variant)
                    .on_click(move |_event, window, cx| {
                        if let Some(ref cb) = confirm_action {
                            cb(window, cx);
                        }
                    }),
            );

        let card = div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap(px(14.0))
            .w(px(440.0))
            .max_w(px(480.0))
            .p(px(20.0))
            .rounded(px(10.0))
            .bg(theme.card_bg)
            .border_1()
            .border_color(theme.card_border)
            .shadow_xl()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            })
            .child(header)
            .child(body)
            .child(footer);

        let card = if cx.reduce_motion() {
            card.into_any_element()
        } else {
            card.with_animation(
                ElementId::Name("modal_enter".into()),
                Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                move |el, delta| {
                    let opacity = delta;
                    let offset_y = (1.0 - delta) * 12.0;
                    el.opacity(opacity).mt(px(offset_y))
                },
            )
            .into_any_element()
        };

        div()
            .id("modal_backdrop")
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x0000_0088))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                if let Some(ref cb) = on_close_backdrop {
                    cb(window, cx);
                }
            })
            .child(card)
    }
}
