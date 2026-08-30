use std::sync::Arc;

use gpui::{App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::features::discord_rpc::DiscordRpcActivity;
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::{Theme, ThemeMode, ThemePalette};
use crate::shared::ui::{Dropdown, GroupCard, Switch};

pub type StringHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type BoolHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type OptionHoverHandler =
    Arc<dyn Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static>;
pub type VoidHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[allow(clippy::struct_excessive_bools)]
#[derive(IntoElement)]
pub struct SettingsPage {
    current_locale: &'static str,
    minimize_to_tray: bool,
    autostart: bool,
    autostart_to_tray: bool,
    discord_rpc: DiscordRpcActivity,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
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
    on_toggle_dropdown: Option<DropdownToggleHandler>,
    on_hover_dropdown: Option<DropdownHoverHandler>,
    on_hover_option: Option<OptionHoverHandler>,
    on_close_dropdowns: Option<VoidHandler>,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new(
            "system",
            false,
            false,
            false,
            DiscordRpcActivity::Disabled,
            None,
            false,
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
        open_dropdown: Option<&'static str>,
        open_dropdown_upward: bool,
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
            open_dropdown,
            open_dropdown_upward,
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
            on_toggle_dropdown: None,
            on_hover_dropdown: None,
            on_hover_option: None,
            on_close_dropdowns: None,
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
        let lang_text = div()
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
                    .child(rust_i18n::t!("settings.language_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.language_desc").to_string()),
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
                .options(vec![
                    (
                        "system",
                        Box::leak(sys_lang_label.into_boxed_str()),
                        Some("icons/languages.svg"),
                    ),
                    ("ru", "Русский", Some("icons/flags/ru.png")),
                    ("en", "English", Some("icons/flags/us.png")),
                ])
                .open(open_dropdown == Some("language"))
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

        let language_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(lang_text)
            .child(language_dropdown);

        // 2. Theme Row (Text stack exactly matched to 32px height of select trigger)
        let theme_text = div()
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
                    .child(rust_i18n::t!("settings.theme_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.theme_desc").to_string()),
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
                .options(vec![
                    (
                        "system",
                        Box::leak(sys_th_label.into_boxed_str()),
                        Some("icons/monitor.svg"),
                    ),
                    (
                        "dark",
                        Box::leak(dark_label.into_boxed_str()),
                        Some("icons/moon.svg"),
                    ),
                    (
                        "light",
                        Box::leak(light_label.into_boxed_str()),
                        Some("icons/sun.svg"),
                    ),
                ])
                .open(open_dropdown == Some("theme"))
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

        let theme_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(theme_text)
            .child(theme_dropdown);

        // 3. Palette Row (Text stack exactly matched to 32px height of select trigger)
        let pal_text = div()
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
                    .child(rust_i18n::t!("settings.palette_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.palette_desc").to_string()),
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
                .options(vec![
                    (
                        "arclate",
                        Box::leak(arclate_label.into_boxed_str()),
                        Some("icons/palette.svg"),
                    ),
                    (
                        "flexoki",
                        Box::leak(flexoki_label.into_boxed_str()),
                        Some("icons/palette.svg"),
                    ),
                ])
                .open(open_dropdown == Some("palette"))
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

        let palette_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(pal_text)
            .child(palette_dropdown);

        // 4. Transparency Row (Text stack exactly matched to 32px height of Switch)
        let trans_text = div()
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
                    .child(rust_i18n::t!("settings.transparency_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.transparency_desc").to_string()),
            );

        let switch_el = Switch::new("transparency_switch", theme.transparency).on_toggle(
            move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_trans {
                    h(new_val, window, cx);
                }
            },
        );

        let transparency_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(trans_text)
            .child(switch_el);

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
        let min_tray_text = div()
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
                    .child(rust_i18n::t!("settings.minimize_to_tray_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.minimize_to_tray_desc").to_string()),
            );

        let min_tray_switch = Switch::new("min_tray_switch", minimize_to_tray).on_toggle(
            move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_min_tray {
                    h(new_val, window, cx);
                }
            },
        );

        let min_tray_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(min_tray_text)
            .child(min_tray_switch);

        // 2. Autostart on logon row
        let autostart_text = div()
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
                    .child(rust_i18n::t!("settings.autostart_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.autostart_desc").to_string()),
            );

        let autostart_switch =
            Switch::new("autostart_switch", autostart).on_toggle(move |new_val, window, cx| {
                if let Some(ref h) = on_toggle_autostart {
                    h(new_val, window, cx);
                }
            });

        let autostart_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(autostart_text)
            .child(autostart_switch);

        // 3. Autostart to tray row (shown conditionally when autostart is enabled)
        let autostart_to_tray_row = if autostart {
            let autostart_tray_text = div()
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
                        .child(rust_i18n::t!("settings.autostart_to_tray_title").to_string()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .line_height(px(14.0))
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_muted)
                        .child(rust_i18n::t!("settings.autostart_to_tray_desc").to_string()),
                );

            let autostart_tray_switch = Switch::new("autostart_tray_switch", autostart_to_tray)
                .on_toggle(move |new_val, window, cx| {
                    if let Some(ref h) = on_toggle_autostart_tray {
                        h(new_val, window, cx);
                    }
                });

            Some(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(autostart_tray_text)
                    .child(autostart_tray_switch),
            )
        } else {
            None
        };

        // 4. Discord Rich Presence row
        let discord_text = div()
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
                    .child(rust_i18n::t!("settings.discord_rpc_title").to_string()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(rust_i18n::t!("settings.discord_rpc_desc").to_string()),
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
        .options(vec![
            (
                "disabled",
                Box::leak(disc_dis_label.into_boxed_str()),
                Some(DiscordRpcActivity::Disabled.icon()),
            ),
            (
                "playing",
                Box::leak(disc_play_label.into_boxed_str()),
                Some(DiscordRpcActivity::Playing.icon()),
            ),
            (
                "listening",
                Box::leak(disc_list_label.into_boxed_str()),
                Some(DiscordRpcActivity::Listening.icon()),
            ),
            (
                "watching",
                Box::leak(disc_watch_label.into_boxed_str()),
                Some(DiscordRpcActivity::Watching.icon()),
            ),
            (
                "competing",
                Box::leak(disc_comp_label.into_boxed_str()),
                Some(DiscordRpcActivity::Competing.icon()),
            ),
        ])
        .open(open_dropdown == Some("discord"))
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

        let discord_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(discord_text)
            .child(discord_dropdown);

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

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
            .child(appearance_card)
            .child(behavior_card)
    }
}
