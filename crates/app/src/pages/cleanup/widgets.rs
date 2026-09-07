use std::rc::Rc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ClickEvent, ElementId, FontWeight,
    InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
    div, ease_in_out, linear_color_stop, linear_gradient, px,
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
        .variant(BadgeVariant::Secondary)
        .into_any_element()
}

pub fn checkbox(
    id: String,
    checked: bool,
    enabled: bool,
    theme: &Theme,
    on_toggle: TargetHandler,
) -> AnyElement {
    let theme = *theme;
    let click_id = id.clone();
    div()
        .id(ElementId::Name(format!("cb_{id}").into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .rounded(px(5.0))
        .border_1()
        .when(checked, |d| {
            d.bg(theme.accent_blue).border_color(theme.accent_blue)
        })
        .when(!checked, |d| {
            d.bg(theme.input_bg).border_color(theme.card_border)
        })
        .when(enabled, |d| {
            d.cursor_pointer().on_click(move |_event, window, cx| {
                cx.stop_propagation();
                on_toggle(click_id.clone(), window, cx);
            })
        })
        .when(!enabled, |d| d.opacity(0.45))
        .when(checked, |d| {
            d.child(
                Icon::new("icons/check.svg")
                    .size(px(12.0))
                    .color(theme.selected_text),
            )
        })
        .into_any_element()
}

#[allow(dead_code)]
pub fn status_pill(label: String, active: bool, theme: &Theme) -> AnyElement {
    let theme = *theme;
    let (bg, text, dot) = if active {
        (
            theme.accent_green.opacity(0.12),
            theme.accent_green,
            theme.accent_green,
        )
    } else {
        (
            theme.text_muted.opacity(0.12),
            theme.text_muted,
            theme.text_muted,
        )
    };

    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(9.0))
        .py(px(4.0))
        .rounded(px(999.0))
        .bg(bg)
        .child(div().size(px(6.0)).rounded(px(999.0)).bg(dot))
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(text)
                .child(label),
        )
        .into_any_element()
}

#[allow(dead_code)]
pub fn header_checkbox(
    id: String,
    checked: bool,
    enabled: bool,
    theme: &Theme,
    on_toggle: ClickHandler,
) -> AnyElement {
    let theme = *theme;
    div()
        .id(ElementId::Name(format!("hcb_{id}").into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .rounded(px(5.0))
        .border_1()
        .when(checked, |d| {
            d.bg(theme.accent_blue).border_color(theme.accent_blue)
        })
        .when(!checked, |d| {
            d.bg(theme.input_bg).border_color(theme.card_border)
        })
        .when(checked, |d| {
            d.child(
                Icon::new("icons/check.svg")
                    .size(px(12.0))
                    .color(theme.selected_text),
            )
        })
        .when(!enabled, |d| d.opacity(0.45))
        .when(enabled, |d| {
            d.cursor_pointer().on_hover(move |hovered, _window, _cx| {
                let _ = hovered;
            })
        })
        .when(enabled, |d| {
            d.on_click(move |event, window, cx| {
                cx.stop_propagation();
                on_toggle(event, window, cx);
            })
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
    let variant = if enabled {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Outline
    };
    let mut button = Button::new(id, label)
        .size(ButtonSize::Md)
        .variant(variant)
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

#[derive(Clone, Copy)]
pub struct CardProps<'a> {
    pub category_id: &'static str,
    pub idx: usize,
    pub scanning: bool,
    pub cleaning: bool,
    pub recently_cleaned: bool,
    pub has_targets: bool,
    pub scanned: bool,
    pub reduce_motion: bool,
    pub theme: &'a Theme,
}

#[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
pub fn render_card(
    props: CardProps<'_>,
    header: impl IntoElement,
    body: impl IntoElement,
) -> AnyElement {
    let theme = *props.theme;
    let category_id = props.category_id;
    if (props.scanning || props.cleaning) && !props.reduce_motion {
        let is_clean = props.cleaning;
        let anim_id = if is_clean {
            format!("cleanup_clean_wave_{category_id}")
        } else {
            format!("cleanup_scan_wave_{category_id}")
        };
        let phase_offset = (props.idx as f32) * 51.4;
        let accent_color = if is_clean {
            theme.accent_orange
        } else {
            theme.accent_blue
        };
        let card_inner = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(9.0))
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body);

        div()
            .id(ElementId::Name(
                format!("cleanup_card_{category_id}").into(),
            ))
            .flex()
            .w_full()
            .rounded(px(10.0))
            .p(px(1.0))
            .overflow_hidden()
            .with_animation(
                ElementId::Name(anim_id.into()),
                Animation::new(Duration::from_millis(2400)).repeat(),
                move |wrap, delta| {
                    let angle = (delta * 360.0 + phase_offset) % 360.0;
                    let pulse =
                        0.75 + 0.25 * ((delta * std::f32::consts::TAU * 2.0 + phase_offset).sin());
                    let wave_bg = linear_gradient(
                        angle,
                        linear_color_stop(accent_color.opacity(pulse), 0.0),
                        linear_color_stop(theme.card_border.opacity(0.30), 1.0),
                    );
                    wrap.bg(wave_bg)
                },
            )
            .child(card_inner)
            .into_any_element()
    } else if props.recently_cleaned && !props.reduce_motion {
        let anim_id = format!("cleanup_fade_{category_id}");
        let card_inner = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(9.0))
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body);

        div()
            .id(ElementId::Name(
                format!("cleanup_card_{category_id}").into(),
            ))
            .flex()
            .w_full()
            .rounded(px(10.0))
            .p(px(1.0))
            .overflow_hidden()
            .with_animation(
                ElementId::Name(anim_id.into()),
                Animation::new(Duration::from_millis(2000)).with_easing(ease_in_out),
                move |wrap, delta| {
                    let fade = (1.0 - delta).clamp(0.0, 1.0);
                    let border_color = if fade > 0.02 {
                        theme.accent_green.opacity(fade)
                    } else {
                        theme.card_border
                    };
                    wrap.bg(border_color)
                },
            )
            .child(card_inner)
            .into_any_element()
    } else {
        let border_color = if props.recently_cleaned {
            theme.accent_green
        } else if props.cleaning {
            theme.accent_orange
        } else if props.scanning {
            theme.accent_blue.opacity(0.6)
        } else if props.has_targets && props.scanned {
            theme.accent_blue
        } else {
            theme.card_border
        };
        div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(10.0))
            .border_1()
            .border_color(border_color)
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body)
            .into_any_element()
    }
}
