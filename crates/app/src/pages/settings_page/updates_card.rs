use gpui::{ElementId, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div, px};

use crate::features::updater::{CURRENT_VERSION, UpdateState};
use crate::shared::theme::Theme;
use crate::shared::ui::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, GroupCard, IconButton,
    IconButtonVariant, Switch, TooltipState,
};

use super::types::*;

pub(crate) struct UpdatesCardParams<'a> {
    pub theme: &'a Theme,
    pub update_state: &'a UpdateState,
    pub check_updates: bool,
    pub on_toggle_check_updates: Option<BoolHandler>,
    pub on_check_update: Option<VoidHandler>,
    pub on_download_and_install_update: Option<VoidHandler>,
    pub on_hover_tooltip: Option<TooltipHoverHandler>,
}

pub(crate) fn build_updates_card(params: UpdatesCardParams<'_>) -> GroupCard {
    let theme = params.theme;
    let current_version_label = format!("v{CURRENT_VERSION}");
    let version_badge = Badge::new("settings_current_version_badge", current_version_label)
        .variant(BadgeVariant::Neutral);

    let (status_label, status_variant, status_icon) = match params.update_state {
        UpdateState::Idle | UpdateState::UpToDate => (
            rust_i18n::t!("settings.status_latest").to_string(),
            BadgeVariant::Success,
            Some("icons/check.svg"),
        ),
        UpdateState::Checking => (
            rust_i18n::t!("settings.status_checking").to_string(),
            BadgeVariant::Neutral,
            Some("icons/refresh-cw.svg"),
        ),
        UpdateState::UpdateAvailable(info) => (
            format!(
                "{} v{}",
                rust_i18n::t!("settings.status_has_update"),
                info.version
            ),
            BadgeVariant::Accent,
            Some("icons/arrow-up.svg"),
        ),
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        UpdateState::Downloading { progress, .. } => (
            rust_i18n::t!(
                "settings.downloading_btn",
                percent = (*progress * 100.0) as u32
            )
            .to_string(),
            BadgeVariant::Accent,
            Some("icons/download.svg"),
        ),
        UpdateState::Installing { .. } => (
            rust_i18n::t!("settings.update_installing").to_string(),
            BadgeVariant::Accent,
            Some("icons/loader.svg"),
        ),
        UpdateState::Error(_) => (
            rust_i18n::t!("settings.status_error").to_string(),
            BadgeVariant::Warning,
            Some("icons/alert-circle.svg"),
        ),
    };

    let mut status_badge =
        Badge::new("settings_status_badge", status_label).variant(status_variant);
    if let Some(ic) = status_icon {
        status_badge = status_badge.icon(ic);
    }

    let title_badges = div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(version_badge)
        .child(status_badge);

    let is_checking = matches!(params.update_state, UpdateState::Checking);
    let is_downloading = matches!(
        params.update_state,
        UpdateState::Downloading { .. } | UpdateState::Installing { .. }
    );

    let mut header_actions = div().flex().items_center().gap(px(8.0));

    if let UpdateState::UpdateAvailable(_) = params.update_state {
        let on_dl = params.on_download_and_install_update.clone();
        let install_btn = Button::new(
            "settings_install_update_btn",
            rust_i18n::t!("settings.download_and_install_btn").to_string(),
        )
        .size(ButtonSize::Md)
        .variant(ButtonVariant::Primary)
        .icon_left("icons/download.svg")
        .on_click(move |_, window, cx| {
            if let Some(ref h) = on_dl {
                h(window, cx);
            }
        });
        header_actions = header_actions.child(install_btn);
    } else if let UpdateState::Downloading { progress, .. } = params.update_state {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let dl_label = rust_i18n::t!(
            "settings.downloading_btn",
            percent = (progress * 100.0) as u32
        )
        .to_string();
        let dl_btn = Button::new("settings_downloading_btn", dl_label)
            .size(ButtonSize::Md)
            .variant(ButtonVariant::Primary)
            .icon_left("icons/download.svg")
            .disabled(true);
        header_actions = header_actions.child(dl_btn);
    }

    let on_check = params.on_check_update.clone();
    let check_btn = IconButton::new("settings_check_update_btn", "icons/refresh-cw.svg")
        .variant(IconButtonVariant::Outline)
        .loading(is_checking)
        .disabled(is_checking || is_downloading)
        .on_click(move |_, window, cx| {
            if let Some(ref h) = on_check {
                h(window, cx);
            }
        });

    let tt_handler = params.on_hover_tooltip.clone();
    let tooltip_msg = if is_checking {
        rust_i18n::t!("settings.status_checking").to_string()
    } else {
        rust_i18n::t!("settings.check_updates_tooltip").to_string()
    };

    let check_btn_wrapper = div()
        .id(ElementId::Name(
            "settings_check_update_wrapper".into(),
        ))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_move({
            let tt_h = tt_handler.clone();
            let tt_text = tooltip_msg.clone();
            move |event, window, cx| {
                if let Some(ref h) = tt_h {
                    h(
                        Some(TooltipState {
                            text: tt_text.clone().into(),
                            cursor_pos: event.position,
                        }),
                        window,
                        cx,
                    );
                }
            }
        })
        .on_hover({
            let tt_h = tt_handler;
            move |hovered, window, cx| {
                if !hovered {
                    if let Some(ref h) = tt_h {
                        h(None, window, cx);
                    }
                }
            }
        })
        .child(check_btn);

    header_actions = header_actions.child(check_btn_wrapper);

    let auto_check_text = settings_row_text(
        rust_i18n::t!("settings.auto_check_updates_title"),
        rust_i18n::t!("settings.auto_check_updates_desc"),
        theme,
    );

    let on_toggle_check = params.on_toggle_check_updates.clone();
    let auto_check_switch = Switch::new("auto_check_switch", params.check_updates).on_toggle(
        move |new_val, window, cx| {
            if let Some(ref h) = on_toggle_check {
                h(new_val, window, cx);
            }
        },
    );

    let auto_check_row = settings_row(auto_check_text, auto_check_switch);

    GroupCard::new(
        "icons/download.svg",
        rust_i18n::t!("settings.updates_title").to_string(),
        rust_i18n::t!("settings.updates_desc").to_string(),
    )
    .icon_color(theme.accent_blue)
    .title_accessory(title_badges)
    .header_action(header_actions)
    .child(auto_check_row)
}