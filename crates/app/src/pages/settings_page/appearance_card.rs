use crate::shared::theme::{Theme, ThemeMode, ThemePalette};
use crate::shared::ui::{Dropdown, GroupCard, Switch};

use super::types::*;

pub(crate) struct AppearanceCardParams<'a> {
    pub theme: &'a Theme,
    pub current_locale: &'static str,
    pub open_dropdown: Option<&'static str>,
    pub opening_dropdown: Option<&'static str>,
    pub closing_dropdown: Option<&'static str>,
    pub open_dropdown_upward: bool,
    pub hovered_dropdown: Option<&'static str>,
    pub hovered_option: Option<(&'static str, &'static str)>,
    pub pending_selection: Option<(&'static str, &'static str)>,
    pub on_change_palette: Option<StringHandler>,
    pub on_change_language: Option<StringHandler>,
    pub on_change_theme: Option<StringHandler>,
    pub on_change_transparency: Option<BoolHandler>,
    pub on_toggle_dropdown: Option<DropdownToggleHandler>,
    pub on_hover_dropdown: Option<DropdownHoverHandler>,
    pub on_hover_option: Option<OptionHoverHandler>,
    pub on_close_dropdowns: Option<VoidHandler>,
}

pub(crate) fn build_appearance_card(params: AppearanceCardParams<'_>) -> GroupCard {
    let theme = params.theme;
    let current_locale = params.current_locale;
    let open_dropdown = params.open_dropdown;
    let open_dropdown_upward = params.open_dropdown_upward;
    let opening_dropdown = params.opening_dropdown;
    let closing_dropdown = params.closing_dropdown;
    let hovered_dropdown = params.hovered_dropdown;
    let hovered_option = params.hovered_option;
    let pending_selection = params.pending_selection;

    let on_change_pal = params.on_change_palette;
    let on_change_lang = params.on_change_language;
    let on_change_th = params.on_change_theme;
    let on_toggle_trans = params.on_change_transparency;

    let on_toggle_pal = params.on_toggle_dropdown.clone();
    let on_toggle_lang = params.on_toggle_dropdown.clone();
    let on_toggle_th = params.on_toggle_dropdown;

    let on_hover_pal = params.on_hover_dropdown.clone();
    let on_hover_th = params.on_hover_dropdown.clone();
    let on_hover_lang = params.on_hover_dropdown;

    let on_hover_opt_pal = params.on_hover_option.clone();
    let on_hover_opt_th = params.on_hover_option.clone();
    let on_hover_opt_lang = params.on_hover_option;

    let on_close_pal = params.on_close_dropdowns.clone();
    let on_close_th = params.on_close_dropdowns.clone();
    let on_close_lang = params.on_close_dropdowns;

    // 1. Language Row
    let lang_text = settings_row_text(
        rust_i18n::t!("settings.language_title"),
        rust_i18n::t!("settings.language_desc"),
        theme,
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

    // 2. Theme Row
    let theme_text = settings_row_text(
        rust_i18n::t!("settings.theme_title"),
        rust_i18n::t!("settings.theme_desc"),
        theme,
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

    // 3. Palette Row
    let pal_text = settings_row_text(
        rust_i18n::t!("settings.palette_title"),
        rust_i18n::t!("settings.palette_desc"),
        theme,
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

    // 4. Transparency Row
    let trans_text = settings_row_text(
        rust_i18n::t!("settings.transparency_title"),
        rust_i18n::t!("settings.transparency_desc"),
        theme,
    );

    let switch_el = Switch::new("transparency_switch", theme.transparency).on_toggle(
        move |new_val, window, cx| {
            if let Some(ref h) = on_toggle_trans {
                h(new_val, window, cx);
            }
        },
    );

    let transparency_row = settings_row(trans_text, switch_el);

    GroupCard::new(
        "icons/palette.svg",
        rust_i18n::t!("settings.appearance_title").to_string(),
        rust_i18n::t!("settings.appearance_desc").to_string(),
    )
    .icon_color(theme.accent_blue)
    .child(language_row)
    .child(theme_row)
    .child(palette_row)
    .child(transparency_row)
}