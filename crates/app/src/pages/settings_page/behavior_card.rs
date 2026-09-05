use crate::features::discord_rpc::DiscordRpcActivity;
use crate::shared::theme::Theme;
use crate::shared::ui::{Dropdown, GroupCard, Switch};

use super::types::*;

pub(crate) struct BehaviorCardParams<'a> {
    pub theme: &'a Theme,
    pub minimize_to_tray: bool,
    pub autostart: bool,
    pub autostart_to_tray: bool,
    pub discord_rpc: DiscordRpcActivity,
    pub open_dropdown: Option<&'static str>,
    pub opening_dropdown: Option<&'static str>,
    pub closing_dropdown: Option<&'static str>,
    pub open_dropdown_upward: bool,
    pub hovered_dropdown: Option<&'static str>,
    pub hovered_option: Option<(&'static str, &'static str)>,
    pub pending_selection: Option<(&'static str, &'static str)>,
    pub on_toggle_minimize_to_tray: Option<BoolHandler>,
    pub on_toggle_autostart: Option<BoolHandler>,
    pub on_toggle_autostart_to_tray: Option<BoolHandler>,
    pub on_change_discord_rpc: Option<StringHandler>,
    pub on_toggle_dropdown: Option<DropdownToggleHandler>,
    pub on_hover_dropdown: Option<DropdownHoverHandler>,
    pub on_hover_option: Option<OptionHoverHandler>,
    pub on_close_dropdowns: Option<VoidHandler>,
}

pub(crate) fn build_behavior_card(params: BehaviorCardParams<'_>) -> GroupCard {
    let theme = params.theme;
    let minimize_to_tray = params.minimize_to_tray;
    let autostart = params.autostart;
    let autostart_to_tray = params.autostart_to_tray;
    let discord_rpc = params.discord_rpc;
    let open_dropdown = params.open_dropdown;
    let opening_dropdown = params.opening_dropdown;
    let closing_dropdown = params.closing_dropdown;
    let open_dropdown_upward = params.open_dropdown_upward;
    let hovered_dropdown = params.hovered_dropdown;
    let hovered_option = params.hovered_option;
    let pending_selection = params.pending_selection;

    let on_toggle_min_tray = params.on_toggle_minimize_to_tray;
    let on_toggle_autostart = params.on_toggle_autostart;
    let on_toggle_autostart_tray = params.on_toggle_autostart_to_tray;
    let on_change_disc = params.on_change_discord_rpc;
    let on_toggle_disc = params.on_toggle_dropdown;
    let on_hover_disc = params.on_hover_dropdown;
    let on_hover_opt_disc = params.on_hover_option;
    let on_close_disc = params.on_close_dropdowns;

    // 1. Minimize to tray row
    let min_tray_text = settings_row_text(
        rust_i18n::t!("settings.minimize_to_tray_title"),
        rust_i18n::t!("settings.minimize_to_tray_desc"),
        theme,
    );

    let min_tray_switch = Switch::new("min_tray_switch", minimize_to_tray).on_toggle(
        move |new_val, window, cx| {
            if let Some(ref h) = on_toggle_min_tray {
                h(new_val, window, cx);
            }
        },
    );

    let min_tray_row = settings_row(min_tray_text, min_tray_switch);

    // 2. Autostart on logon row
    let autostart_text = settings_row_text(
        rust_i18n::t!("settings.autostart_title"),
        rust_i18n::t!("settings.autostart_desc"),
        theme,
    );

    let autostart_switch =
        Switch::new("autostart_switch", autostart).on_toggle(move |new_val, window, cx| {
            if let Some(ref h) = on_toggle_autostart {
                h(new_val, window, cx);
            }
        });

    let autostart_row = settings_row(autostart_text, autostart_switch);

    // 3. Autostart to tray row (shown conditionally when autostart is enabled)
    let autostart_to_tray_row = if autostart {
        let autostart_tray_text = settings_row_text(
            rust_i18n::t!("settings.autostart_to_tray_title"),
            rust_i18n::t!("settings.autostart_to_tray_desc"),
            theme,
        );

        let autostart_tray_switch = Switch::new("autostart_tray_switch", autostart_to_tray)
            .on_toggle(move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_autostart_tray {
                    h(new_val, window, cx);
                }
            });

        Some(settings_row(autostart_tray_text, autostart_tray_switch))
    } else {
        None
    };

    // 4. Discord Rich Presence row
    let discord_text = settings_row_text(
        rust_i18n::t!("settings.discord_rpc_title"),
        rust_i18n::t!("settings.discord_rpc_desc"),
        theme,
    );

    let is_discord_morphing = pending_selection.map(|(d, _)| d) == Some("discord");
    let effective_discord_code = if let Some(("discord", val)) = pending_selection {
        val
    } else {
        discord_rpc.as_str()
    };

    let effective_discord_activity = DiscordRpcActivity::from_str(effective_discord_code);
    let effective_discord_label =
        rust_i18n::t!(effective_discord_activity.label_key()).to_string();
    let discord_icon = effective_discord_activity.icon();

    let disc_dis_label = rust_i18n::t!("settings.discord_disabled").to_string();
    let disc_play_label = rust_i18n::t!("settings.discord_playing").to_string();
    let disc_list_label = rust_i18n::t!("settings.discord_listening").to_string();
    let disc_watch_label = rust_i18n::t!("settings.discord_watching").to_string();
    let disc_comp_label = rust_i18n::t!("settings.discord_competing").to_string();

    let discord_hovered_opt = if let Some(("discord", val)) = hovered_option {
        Some(val)
    } else {
        None
    };

    let discord_dropdown = Dropdown::new(
        "discord_select",
        effective_discord_label,
        effective_discord_code,
    )
    .icon(discord_icon)
    .localized_options(vec![
        (
            "disabled",
            disc_dis_label.into(),
            Some(DiscordRpcActivity::Disabled.icon()),
        ),
        (
            "playing",
            disc_play_label.into(),
            Some(DiscordRpcActivity::Playing.icon()),
        ),
        (
            "listening",
            disc_list_label.into(),
            Some(DiscordRpcActivity::Listening.icon()),
        ),
        (
            "watching",
            disc_watch_label.into(),
            Some(DiscordRpcActivity::Watching.icon()),
        ),
        (
            "competing",
            disc_comp_label.into(),
            Some(DiscordRpcActivity::Competing.icon()),
        ),
    ])
    .open(open_dropdown == Some("discord"))
    .opening(opening_dropdown == Some("discord"))
    .closing(closing_dropdown == Some("discord"))
    .upward(open_dropdown_upward)
    .morphing(is_discord_morphing)
    .hovered(hovered_dropdown == Some("discord"))
    .hovered_option(discord_hovered_opt)
    .on_hover_trigger(move |hovered, window, cx| {
        if let Some(ref h) = on_hover_disc {
            h("discord", hovered, window, cx);
        }
    })
    .on_hover_option(move |opt_val, &hov, window, cx| {
        if let Some(ref h) = on_hover_opt_disc {
            let static_val = match opt_val {
                "playing" => "playing",
                "listening" => "listening",
                "watching" => "watching",
                "competing" => "competing",
                _ => "disabled",
            };
            h("discord", static_val, &hov, window, cx);
        }
    })
    .on_toggle(move |window, cx| {
        if let Some(ref h) = on_toggle_disc {
            h("discord", window, cx);
        }
    })
    .on_select(move |code, window, cx| {
        if let Some(ref h) = on_change_disc {
            h(code, window, cx);
        }
    })
    .on_close(move |window, cx| {
        if let Some(ref h) = on_close_disc {
            h(window, cx);
        }
    });

    let discord_row = settings_row(discord_text, discord_dropdown);

    let mut behavior_card = GroupCard::new(
        "icons/sliders-horizontal.svg",
        rust_i18n::t!("settings.behavior_title").to_string(),
        rust_i18n::t!("settings.behavior_desc").to_string(),
    )
    .icon_color(theme.accent_blue)
    .child(min_tray_row)
    .child(autostart_row);

    if let Some(tray_row) = autostart_to_tray_row {
        behavior_card = behavior_card.child(tray_row);
    }

    behavior_card.child(discord_row)
}