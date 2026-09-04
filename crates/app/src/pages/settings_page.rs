use std::sync::Arc;

use gpui::{
    App, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::features::discord_rpc::DiscordRpcActivity;
use crate::features::navigation::AppRoute;
use crate::features::updater::{CURRENT_VERSION, UpdateState};
use crate::pages::page_header::PageHeader;
use crate::shared::theme::{Theme, ThemeMode, ThemePalette};
use crate::shared::ui::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Dropdown, GroupCard, IconButton,
    IconButtonVariant, Switch, TooltipState,
};

pub type StringHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type BoolHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type OptionHoverHandler =
    Arc<dyn Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static>;
pub type VoidHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;

#[allow(clippy::struct_excessive_bools)]
#[derive(IntoElement)]
pub struct SettingsPage {
    current_locale: &'static str,
    minimize_to_tray: bool,
    autostart: bool,
    autostart_to_tray: bool,
    discord_rpc: DiscordRpcActivity,
    check_updates: bool,
    update_state: UpdateState,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    opening_dropdown: Option<&'static str>,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    on_change_palette: Option<StringHandler>,
    on_change_language: Option<StringHandler>,
    on_change_theme: Option<StringHandler>,
    on_change_transparency: Option<BoolHandler>,
    on_toggle_minimize_to_tray: Option<BoolHandler>,
    on_toggle_autostart: Option<BoolHandler>,
    on_toggle_autostart_to_tray: Option<BoolHandler>,
    on_change_discord_rpc: Option<StringHandler>,
    on_toggle_check_updates: Option<BoolHandler>,
    on_check_update: Option<VoidHandler>,
    on_download_and_install_update: Option<VoidHandler>,
    on_toggle_dropdown: Option<DropdownToggleHandler>,
    on_hover_dropdown: Option<DropdownHoverHandler>,
    on_hover_option: Option<OptionHoverHandler>,
    on_close_dropdowns: Option<VoidHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new(
            "system",
            false,
            false,
            false,
            DiscordRpcActivity::Disabled,
            true,
            UpdateState::Idle,
            None,
            false,
            None,
            None,
            None,
            None,
            None,
        )
    }
}

impl SettingsPage {
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        current_locale: &'static str,
        minimize_to_tray: bool,
        autostart: bool,
        autostart_to_tray: bool,
        discord_rpc: DiscordRpcActivity,
        check_updates: bool,
        update_state: UpdateState,
        open_dropdown: Option<&'static str>,
        open_dropdown_upward: bool,
        opening_dropdown: Option<&'static str>,
        closing_dropdown: Option<&'static str>,
        hovered_dropdown: Option<&'static str>,
        hovered_option: Option<(&'static str, &'static str)>,
        pending_selection: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            current_locale,
            minimize_to_tray,
            autostart,
            autostart_to_tray,
            discord_rpc,
            check_updates,
            update_state,
            open_dropdown,
            open_dropdown_upward,
            opening_dropdown,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            pending_selection,
            on_change_palette: None,
            on_change_language: None,
            on_change_theme: None,
            on_change_transparency: None,
            on_toggle_minimize_to_tray: None,
            on_toggle_autostart: None,
            on_toggle_autostart_to_tray: None,
            on_change_discord_rpc: None,
            on_toggle_check_updates: None,
            on_check_update: None,
            on_download_and_install_update: None,
            on_toggle_dropdown: None,
            on_hover_dropdown: None,
            on_hover_option: None,
            on_close_dropdowns: None,
            on_hover_tooltip: None,
        }
    }

    #[must_use]
    pub fn on_change_palette(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_palette = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_language(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_language = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_theme(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_theme = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_transparency(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_transparency = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_minimize_to_tray(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_minimize_to_tray = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_autostart(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_autostart = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_autostart_to_tray(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_autostart_to_tray = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_discord_rpc(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_discord_rpc = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_check_updates(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_check_updates = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_check_update(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_check_update = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_download_and_install_update(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_download_and_install_update = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_dropdown(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_dropdown(
        mut self,
        handler: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_option(
        mut self,
        handler: impl Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_option = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close_dropdowns(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close_dropdowns = Some(Arc::new(handler));
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

impl RenderOnce for SettingsPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Settings;
        let current_locale = self.current_locale;
        let minimize_to_tray = self.minimize_to_tray;
        let autostart = self.autostart;
        let autostart_to_tray = self.autostart_to_tray;
        let discord_rpc = self.discord_rpc;

        let open_dropdown = self.open_dropdown;
        let open_dropdown_upward = self.open_dropdown_upward;
        let opening_dropdown = self.opening_dropdown;
        let closing_dropdown = self.closing_dropdown;
        let hovered_dropdown = self.hovered_dropdown;
        let hovered_option = self.hovered_option;
        let pending_selection = self.pending_selection;

        let on_change_pal = self.on_change_palette;
        let on_change_lang = self.on_change_language;
        let on_change_th = self.on_change_theme;
        let on_toggle_trans = self.on_change_transparency;
        let on_toggle_min_tray = self.on_toggle_minimize_to_tray;
        let on_toggle_autostart = self.on_toggle_autostart;
        let on_toggle_autostart_tray = self.on_toggle_autostart_to_tray;
        let on_change_disc = self.on_change_discord_rpc;

        let on_toggle_pal = self.on_toggle_dropdown.clone();
        let on_toggle_lang = self.on_toggle_dropdown.clone();
        let on_toggle_th = self.on_toggle_dropdown.clone();
        let on_toggle_disc = self.on_toggle_dropdown;

        let on_hover_pal = self.on_hover_dropdown.clone();
        let on_hover_th = self.on_hover_dropdown.clone();
        let on_hover_lang = self.on_hover_dropdown.clone();
        let on_hover_disc = self.on_hover_dropdown;

        let on_hover_opt_pal = self.on_hover_option.clone();
        let on_hover_opt_th = self.on_hover_option.clone();
        let on_hover_opt_lang = self.on_hover_option.clone();
        let on_hover_opt_disc = self.on_hover_option;

        let on_close_pal = self.on_close_dropdowns.clone();
        let on_close_th = self.on_close_dropdowns.clone();
        let on_close_lang = self.on_close_dropdowns.clone();
        let on_close_disc = self.on_close_dropdowns;

        // 1. Language Row (Text stack exactly matched to 32px height of select trigger)
        let lang_text = settings_row_text(
            rust_i18n::t!("settings.language_title"),
            rust_i18n::t!("settings.language_desc"),
            &theme,
        );

        let (lang_current_label, lang_icon) = match current_locale {
            "ru" => ("Русский".to_string(), "icons/flags/ru.png"),
            "en" => ("English".to_string(), "icons/flags/us.png"),
            _ => (
                rust_i18n::t!("settings.lang_system").to_string(),
                "icons/languages.svg",
            ),
        };

        let is_lang_morphing = pending_selection.map(|(d, _)| d) == Some("language");

        let effective_lang_code = if let Some(("language", val)) = pending_selection {
            val
        } else {
            current_locale
        };

        let sys_lang_label = rust_i18n::t!("settings.lang_system").to_string();

        let lang_hovered_opt = if let Some(("language", val)) = hovered_option {
            Some(val)
        } else {
            None
        };

        let language_dropdown =
            Dropdown::new("lang_select", lang_current_label, effective_lang_code)
                .icon(lang_icon)
                .localized_options(vec![
                    ("system", sys_lang_label.into(), Some("icons/languages.svg")),
                    ("ru", "Русский".into(), Some("icons/flags/ru.png")),
                    ("en", "English".into(), Some("icons/flags/us.png")),
                ])
                .open(open_dropdown == Some("language"))
                .opening(opening_dropdown == Some("language"))
                .closing(closing_dropdown == Some("language"))
                .upward(open_dropdown_upward)
                .morphing(is_lang_morphing)
                .hovered(hovered_dropdown == Some("language"))
                .hovered_option(lang_hovered_opt)
                .on_hover_trigger(move |hovered, window, cx| {
                    if let Some(ref h) = on_hover_lang {
                        h("language", hovered, window, cx);
                    }
                })
                .on_hover_option(move |opt_val, &hov, window, cx| {
                    if let Some(ref h) = on_hover_opt_lang {
                        let static_val = match opt_val {
                            "system" => "system",
                            "en" => "en",
                            _ => "ru",
                        };
                        h("language", static_val, &hov, window, cx);
                    }
                })
                .on_toggle(move |window, cx| {
                    if let Some(ref h) = on_toggle_lang {
                        h("language", window, cx);
                    }
                })
                .on_select(move |lang, window, cx| {
                    if let Some(ref h) = on_change_lang {
                        h(lang, window, cx);
                    }
                })
                .on_close(move |window, cx| {
                    if let Some(ref h) = on_close_lang {
                        h(window, cx);
                    }
                });

        let language_row = settings_row(lang_text, language_dropdown);

        // 2. Theme Row (Text stack exactly matched to 32px height of select trigger)
        let theme_text = settings_row_text(
            rust_i18n::t!("settings.theme_title"),
            rust_i18n::t!("settings.theme_desc"),
            &theme,
        );

        let (theme_current_code, theme_current_label, theme_icon) = match theme.mode {
            ThemeMode::System => (
                "system",
                rust_i18n::t!("settings.theme_system").to_string(),
                "icons/monitor.svg",
            ),
            ThemeMode::Dark => (
                "dark",
                rust_i18n::t!("settings.theme_dark").to_string(),
                "icons/moon.svg",
            ),
            ThemeMode::Light => (
                "light",
                rust_i18n::t!("settings.theme_light").to_string(),
                "icons/sun.svg",
            ),
        };

        let is_theme_morphing = pending_selection.map(|(d, _)| d) == Some("theme");

        let effective_theme_code = if let Some(("theme", val)) = pending_selection {
            val
        } else {
            theme_current_code
        };

        let sys_th_label = rust_i18n::t!("settings.theme_system").to_string();
        let dark_label = rust_i18n::t!("settings.theme_dark").to_string();
        let light_label = rust_i18n::t!("settings.theme_light").to_string();

        let theme_hovered_opt = if let Some(("theme", val)) = hovered_option {
            Some(val)
        } else {
            None
        };

        let theme_dropdown =
            Dropdown::new("theme_select", theme_current_label, effective_theme_code)
                .icon(theme_icon)
                .localized_options(vec![
                    ("system", sys_th_label.into(), Some("icons/monitor.svg")),
                    ("dark", dark_label.into(), Some("icons/moon.svg")),
                    ("light", light_label.into(), Some("icons/sun.svg")),
                ])
                .open(open_dropdown == Some("theme"))
                .opening(opening_dropdown == Some("theme"))
                .closing(closing_dropdown == Some("theme"))
                .upward(open_dropdown_upward)
                .morphing(is_theme_morphing)
                .hovered(hovered_dropdown == Some("theme"))
                .hovered_option(theme_hovered_opt)
                .on_hover_trigger(move |hovered, window, cx| {
                    if let Some(ref h) = on_hover_th {
                        h("theme", hovered, window, cx);
                    }
                })
                .on_hover_option(move |opt_val, &hov, window, cx| {
                    if let Some(ref h) = on_hover_opt_th {
                        let static_val = match opt_val {
                            "system" => "system",
                            "light" => "light",
                            _ => "dark",
                        };
                        h("theme", static_val, &hov, window, cx);
                    }
                })
                .on_toggle(move |window, cx| {
                    if let Some(ref h) = on_toggle_th {
                        h("theme", window, cx);
                    }
                })
                .on_select(move |mode, window, cx| {
                    if let Some(ref h) = on_change_th {
                        h(mode, window, cx);
                    }
                })
                .on_close(move |window, cx| {
                    if let Some(ref h) = on_close_th {
                        h(window, cx);
                    }
                });

        let theme_row = settings_row(theme_text, theme_dropdown);

        // 3. Palette Row (Text stack exactly matched to 32px height of select trigger)
        let pal_text = settings_row_text(
            rust_i18n::t!("settings.palette_title"),
            rust_i18n::t!("settings.palette_desc"),
            &theme,
        );

        let (pal_current_code, pal_current_label, pal_icon) = match theme.palette {
            ThemePalette::Arclate => (
                "arclate",
                rust_i18n::t!("settings.palette_arclate").to_string(),
                "icons/palette.svg",
            ),
            ThemePalette::Flexoki => (
                "flexoki",
                rust_i18n::t!("settings.palette_flexoki").to_string(),
                "icons/palette.svg",
            ),
        };

        let is_pal_morphing = pending_selection.map(|(d, _)| d) == Some("palette");
        let effective_pal_code = if let Some(("palette", val)) = pending_selection {
            val
        } else {
            pal_current_code
        };

        let arclate_label = rust_i18n::t!("settings.palette_arclate").to_string();
        let flexoki_label = rust_i18n::t!("settings.palette_flexoki").to_string();

        let pal_hovered_opt = if let Some(("palette", val)) = hovered_option {
            Some(val)
        } else {
            None
        };

        let palette_dropdown =
            Dropdown::new("palette_select", pal_current_label, effective_pal_code)
                .icon(pal_icon)
                .localized_options(vec![
                    ("arclate", arclate_label.into(), Some("icons/palette.svg")),
                    ("flexoki", flexoki_label.into(), Some("icons/palette.svg")),
                ])
                .open(open_dropdown == Some("palette"))
                .opening(opening_dropdown == Some("palette"))
                .closing(closing_dropdown == Some("palette"))
                .upward(open_dropdown_upward)
                .morphing(is_pal_morphing)
                .hovered(hovered_dropdown == Some("palette"))
                .hovered_option(pal_hovered_opt)
                .on_hover_trigger(move |hovered, window, cx| {
                    if let Some(ref h) = on_hover_pal {
                        h("palette", hovered, window, cx);
                    }
                })
                .on_hover_option(move |opt_val, &hov, window, cx| {
                    if let Some(ref h) = on_hover_opt_pal {
                        let static_val = match opt_val {
                            "flexoki" => "flexoki",
                            _ => "arclate",
                        };
                        h("palette", static_val, &hov, window, cx);
                    }
                })
                .on_toggle(move |window, cx| {
                    if let Some(ref h) = on_toggle_pal {
                        h("palette", window, cx);
                    }
                })
                .on_select(move |palette_name, window, cx| {
                    if let Some(ref h) = on_change_pal {
                        h(palette_name, window, cx);
                    }
                })
                .on_close(move |window, cx| {
                    if let Some(ref h) = on_close_pal {
                        h(window, cx);
                    }
                });

        let palette_row = settings_row(pal_text, palette_dropdown);

        // 4. Transparency Row (Text stack exactly matched to 32px height of Switch)
        let trans_text = settings_row_text(
            rust_i18n::t!("settings.transparency_title"),
            rust_i18n::t!("settings.transparency_desc"),
            &theme,
        );

        let switch_el = Switch::new("transparency_switch", theme.transparency).on_toggle(
            move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_trans {
                    h(new_val, window, cx);
                }
            },
        );

        let transparency_row = settings_row(trans_text, switch_el);

        let appearance_card = GroupCard::new(
            "icons/palette.svg",
            rust_i18n::t!("settings.appearance_title").to_string(),
            rust_i18n::t!("settings.appearance_desc").to_string(),
        )
        .icon_color(theme.accent_blue)
        .child(language_row)
        .child(theme_row)
        .child(palette_row)
        .child(transparency_row);

        // --- BEHAVIOR CARD ---
        // 1. Minimize to tray row
        let min_tray_text = settings_row_text(
            rust_i18n::t!("settings.minimize_to_tray_title"),
            rust_i18n::t!("settings.minimize_to_tray_desc"),
            &theme,
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
            &theme,
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
                &theme,
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
            &theme,
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

        let behavior_card = behavior_card.child(discord_row);

        // --- UPDATES CARD ---
        let current_version_label = format!("v{CURRENT_VERSION}");
        let version_badge = Badge::new("settings_current_version_badge", current_version_label)
            .variant(BadgeVariant::Neutral);

        let (status_label, status_variant, status_icon) = match &self.update_state {
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

        let is_checking = matches!(self.update_state, UpdateState::Checking);
        let is_downloading = matches!(
            self.update_state,
            UpdateState::Downloading { .. } | UpdateState::Installing { .. }
        );

        let mut header_actions = div().flex().items_center().gap(px(8.0));

        if let UpdateState::UpdateAvailable(_) = self.update_state {
            let on_dl = self.on_download_and_install_update.clone();
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
        } else if let UpdateState::Downloading { progress, .. } = self.update_state {
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

        let on_check = self.on_check_update.clone();
        let check_btn = IconButton::new("settings_check_update_btn", "icons/refresh-cw.svg")
            .variant(IconButtonVariant::Outline)
            .loading(is_checking)
            .disabled(is_checking || is_downloading)
            .on_click(move |_, window, cx| {
                if let Some(ref h) = on_check {
                    h(window, cx);
                }
            });

        let tt_handler = self.on_hover_tooltip.clone();
        let tooltip_msg = if is_checking {
            rust_i18n::t!("settings.status_checking").to_string()
        } else {
            rust_i18n::t!("settings.check_updates_tooltip").to_string()
        };

        let check_btn_wrapper = div()
            .id(gpui::ElementId::Name(
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
            &theme,
        );

        let on_toggle_check = self.on_toggle_check_updates.clone();
        let auto_check_switch = Switch::new("auto_check_switch", self.check_updates).on_toggle(
            move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_check {
                    h(new_val, window, cx);
                }
            },
        );

        let auto_check_row = settings_row(auto_check_text, auto_check_switch);

        let updates_card = GroupCard::new(
            "icons/download.svg",
            rust_i18n::t!("settings.updates_title").to_string(),
            rust_i18n::t!("settings.updates_desc").to_string(),
        )
        .icon_color(theme.accent_blue)
        .title_accessory(title_badges)
        .header_action(header_actions)
        .child(auto_check_row);

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
            .child(appearance_card)
            .child(behavior_card)
            .child(updates_card)
    }
}

fn settings_row_text(
    title: impl Into<SharedString>,
    desc: impl Into<SharedString>,
    theme: &Theme,
) -> Div {
    div()
        .flex()
        .flex_col()
        .justify_between()
        .h(px(32.0))
        .child(
            div()
                .text_size(px(13.5))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(11.5))
                .line_height(px(14.0))
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_muted)
                .child(desc.into()),
        )
}

fn settings_row(left: impl IntoElement, right: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .child(left)
        .child(right)
}
