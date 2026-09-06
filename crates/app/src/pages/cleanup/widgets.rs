use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::entities::cleanup::format_bytes;
use crate::shared::theme::Theme;
use crate::shared::ui::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Icon};

pub type TargetHandler = Rc<dyn Fn(String, &mut Window, &mut App)>;
pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct TargetRow {
    pub id: String,
    pub name: String,
    pub secondary: String,
    pub bytes: u64,
    pub selected: bool,
}

pub fn badge(id: String, text: String, _theme: &Theme) -> AnyElement {
    Badge::new(id, text)
        .variant(BadgeVariant::Outline)
        .into_any_element()
}

pub fn checkbox(
    id: String,
    checked: bool,
    enabled: bool,
    theme: &Theme,
    on_click: TargetHandler,
) -> AnyElement {
    let theme = *theme;
    let border_col = if !enabled {
        theme.input_border.opacity(0.35)
    } else if checked {
        theme.accent_blue
    } else {
        theme.input_border
    };
    let bg_col = if !enabled {
        theme.input_bg.opacity(0.35)
    } else if checked {
        theme.accent_blue
    } else {
        theme.input_bg
    };

    let mut element = div()
        .id(ElementId::Name(id.clone().into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .rounded(px(4.0))
        .border_1()
        .border_color(border_col)
        .bg(bg_col);

    if enabled {
        element = element
            .cursor_pointer()
            .hover(move |style| style.border_color(theme.accent_blue))
            .on_click(move |_event, window, cx| {
                cx.stop_propagation();
                on_click(id.clone(), window, cx);
            });
    }

    element
        .when(checked, |el| {
            el.child(
                Icon::new("icons/check.svg")
                    .size(px(10.0))
                    .color(if enabled {
                        theme.selected_text
                    } else {
                        theme.selected_text.opacity(0.4)
                    }),
            )
        })
        .into_any_element()
}

pub fn clean_button(
    id: String,
    label: String,
    enabled: bool,
    _theme: &Theme,
    on_click: Option<ClickHandler>,
) -> AnyElement {
    let mut button = Button::new(id, label)
        .size(ButtonSize::Md)
        .variant(ButtonVariant::Outline)
        .icon_left("icons/trash-2.svg")
        .disabled(!enabled);

    if enabled && let Some(handler) = on_click {
        button = button.on_click(move |event, window, cx| {
            cx.stop_propagation();
            handler(event, window, cx);
        });
    }

    button.into_any_element()
}

pub fn render_target(target: &TargetRow, theme: &Theme, on_toggle: TargetHandler) -> AnyElement {
    let theme = *theme;
    let id = target.id.clone();
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .h(px(50.0))
        .px(px(10.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(theme.card_border)
        .bg(theme.main_bg)
        .child(checkbox(id, target.selected, true, &theme, on_toggle))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded(px(6.0))
                .bg(theme.accent_green.opacity(0.12))
                .child(
                    Icon::new("icons/check.svg")
                        .size(px(13.0))
                        .color(theme.accent_green),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .text_ellipsis()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(target.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(target.secondary.clone()),
                ),
        )
        .child(
            div()
                .text_size(px(11.5))
                .text_color(theme.text_muted)
                .child(format_bytes(target.bytes)),
        )
        .into_any_element()
}
