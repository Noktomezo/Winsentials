use std::time::Duration;

use gpui::{
    Animation, AnimationExt, ElementId, FontWeight, IntoElement, ParentElement, SharedString,
    Styled, div, pulsating_between, px,
};
use rust_i18n::t;

use crate::entities::SystemInfo;
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;

fn render_info_row(
    label: impl Into<SharedString>,
    value_el: impl IntoElement,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .gap(px(8.0))
        .py(px(2.5))
        .child(
            div()
                .flex_none()
                .text_size(px(12.0))
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .justify_end()
                .min_w(px(0.0))
                .child(value_el),
        )
}

fn render_text_val(text: impl Into<SharedString>, theme: &Theme) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_primary)
        .text_ellipsis()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(text.into())
}

fn render_activation_val(is_activated: bool, theme: &Theme) -> impl IntoElement {
    let (color, text_key) = if is_activated {
        (theme.accent_green, "system.activated")
    } else {
        (theme.accent_red, "system.not_activated")
    };

    // Smooth organic breathing pulsation between 40% and 100% opacity
    let pulse_animation = Animation::new(Duration::from_millis(2200))
        .repeat()
        .with_easing(pulsating_between(0.4, 1.0));

    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(t!(text_key))
        .with_animation(
            ElementId::Name("activation_pulse".into()),
            pulse_animation,
            gpui::Styled::opacity,
        )
}

pub(crate) fn render_system_card(info: &SystemInfo, theme: &Theme) -> impl IntoElement {
    let left_col = div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .flex_1()
        .min_w(px(0.0))
        .child(render_info_row(
            t!("system.os_version"),
            render_text_val(info.os_version.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.motherboard"),
            render_text_val(info.motherboard.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.user"),
            render_text_val(info.username.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.architecture"),
            render_text_val(info.architecture.clone(), theme),
            theme,
        ));

    let right_col = div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .flex_1()
        .min_w(px(0.0))
        .child(render_info_row(
            t!("system.build"),
            render_text_val(info.build_number.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.computer_name"),
            render_text_val(info.computer_name.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.tweaks_applied"),
            render_text_val(info.tweaks_applied.clone(), theme),
            theme,
        ))
        .child(render_info_row(
            t!("system.activation"),
            render_activation_val(info.is_activated, theme),
            theme,
        ));

    let grid_row = div()
        .flex()
        .flex_row()
        .gap(px(16.0))
        .w_full()
        .min_w(px(0.0))
        .child(left_col)
        .child(right_col);

    GroupCard::new("icons/monitor.svg", t!("system.title"), t!("system.desc"))
        .icon_color(theme.accent_blue)
        .child(grid_row)
}