use std::sync::Arc;

use gpui::{App, IntoElement, RenderOnce, Window};

use crate::entities::tweaks::TweakCategory;
use crate::entities::tweaks::input::{
    CtfOptimizationPreset, KeyboardRepeatPreset, SnapKeyPreset, current_ctf_preset,
    current_keyboard_repeat_preset, current_snapkey_preset, snapkey_preset_icon,
    snapkey_preset_label,
};
use crate::features::navigation::AppRoute;
use crate::pages::context_menu_page::{render_tweak_cards_for_category, render_tweak_page};
use crate::shared::theme::Theme;
use crate::shared::ui::tweak_card::{TooltipHoverHandler, TweakBadge};
use crate::shared::ui::{Dropdown, TweakDropdownCard};

pub type TweakToggleHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;
pub type PresetSelectHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type DropdownOptionHoverHandler =
    Arc<dyn Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static>;
pub type VoidHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct InputPage {
    windows_build: u32,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    opening_dropdown: Option<&'static str>,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    on_toggle_tweak: Option<TweakToggleHandler>,
    on_select_preset: Option<PresetSelectHandler>,
    on_select_ctf_preset: Option<PresetSelectHandler>,
    on_select_snapkey_preset: Option<PresetSelectHandler>,
    on_toggle_dropdown: Option<DropdownToggleHandler>,
    on_hover_dropdown: Option<DropdownHoverHandler>,
    on_hover_option: Option<DropdownOptionHoverHandler>,
    on_close_dropdowns: Option<VoidHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl InputPage {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        windows_build: u32,
        open_dropdown: Option<&'static str>,
        open_dropdown_upward: bool,
        opening_dropdown: Option<&'static str>,
        closing_dropdown: Option<&'static str>,
        hovered_dropdown: Option<&'static str>,
        hovered_option: Option<(&'static str, &'static str)>,
        pending_selection: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            windows_build,
            open_dropdown,
            open_dropdown_upward,
            opening_dropdown,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            pending_selection,
            on_toggle_tweak: None,
            on_select_preset: None,
            on_select_ctf_preset: None,
            on_select_snapkey_preset: None,
            on_toggle_dropdown: None,
            on_hover_dropdown: None,
            on_hover_option: None,
            on_close_dropdowns: None,
            on_hover_tooltip: None,
        }
    }

    #[must_use]
    pub fn on_toggle_tweak(
        mut self,
        handler: impl Fn(&'static str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_tweak = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_select_preset(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_preset = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_select_ctf_preset(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_ctf_preset = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_select_snapkey_preset(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_snapkey_preset = Some(Arc::new(handler));
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
        handler: impl Fn(Option<crate::shared::ui::TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for InputPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Input;
        let windows_build = self.windows_build;
        let on_toggle = self.on_toggle_tweak;
        let on_hover_tt = self.on_hover_tooltip;

        let mut tweak_items = render_tweak_cards_for_category(
            TweakCategory::Input,
            windows_build,
            &theme,
            on_toggle.as_ref(),
            on_hover_tt.as_ref(),
            cx,
        );

        let dropdown_name = "keyboard_repeat";
        let current_preset = self
            .pending_selection
            .and_then(|(name, value)| {
                (name == dropdown_name)
                    .then(|| KeyboardRepeatPreset::from_id(value))
                    .flatten()
            })
            .unwrap_or_else(current_keyboard_repeat_preset);
        let current_label = keyboard_repeat_preset_label(current_preset);
        let hovered_option = self
            .hovered_option
            .and_then(|(name, option)| (name == dropdown_name).then_some(option));
        let select_preset = self.on_select_preset;
        let select_ctf = self.on_select_ctf_preset;
        let select_snapkey = self.on_select_snapkey_preset;
        let toggle_dropdown = self.on_toggle_dropdown;
        let hover_dropdown = self.on_hover_dropdown;
        let hover_option = self.on_hover_option;
        let close_dropdowns = self.on_close_dropdowns;

        let toggle_dropdown_kr = toggle_dropdown.clone();
        let hover_dropdown_kr = hover_dropdown.clone();
        let hover_option_kr = hover_option.clone();
        let close_dropdowns_kr = close_dropdowns.clone();

        let toggle_dropdown_ctf = toggle_dropdown.clone();
        let hover_dropdown_ctf = hover_dropdown.clone();
        let hover_option_ctf = hover_option.clone();
        let close_dropdowns_ctf = close_dropdowns.clone();

        let toggle_dropdown_sk = toggle_dropdown;
        let hover_dropdown_sk = hover_dropdown;
        let hover_option_sk = hover_option;
        let close_dropdowns_sk = close_dropdowns;

        let on_hover_tt_kr = on_hover_tt.clone();
        let on_hover_tt_ctf = on_hover_tt.clone();
        let on_hover_tt_sk = on_hover_tt;

        let options = KeyboardRepeatPreset::ALL
            .into_iter()
            .map(|preset| {
                (
                    preset.id(),
                    keyboard_repeat_preset_label(preset).into(),
                    Some(keyboard_repeat_preset_icon(preset)),
                )
            })
            .collect();

        let dropdown = Dropdown::new("keyboard_repeat_select", current_label, current_preset.id())
            .icon(keyboard_repeat_preset_icon(current_preset))
            .localized_options(options)
            .open(self.open_dropdown == Some(dropdown_name))
            .opening(self.opening_dropdown == Some(dropdown_name))
            .closing(self.closing_dropdown == Some(dropdown_name))
            .upward(self.open_dropdown_upward)
            .morphing(self.pending_selection.map(|(name, _)| name) == Some(dropdown_name))
            .hovered(self.hovered_dropdown == Some(dropdown_name))
            .hovered_option(hovered_option)
            .on_toggle(move |window, cx| {
                if let Some(ref handler) = toggle_dropdown_kr {
                    handler(dropdown_name, window, cx);
                }
            })
            .on_select(move |value, window, cx| {
                if let Some(ref handler) = select_preset {
                    handler(value, window, cx);
                }
            })
            .on_hover_trigger(move |hovered, window, cx| {
                if let Some(ref handler) = hover_dropdown_kr {
                    handler(dropdown_name, hovered, window, cx);
                }
            })
            .on_hover_option(move |option, hovered, window, cx| {
                if let Some(ref handler) = hover_option_kr {
                    let option = KeyboardRepeatPreset::from_id(option)
                        .unwrap_or(KeyboardRepeatPreset::Standard)
                        .id();
                    handler(dropdown_name, option, hovered, window, cx);
                }
            })
            .on_close(move |window, cx| {
                if let Some(ref handler) = close_dropdowns_kr {
                    handler(window, cx);
                }
            });

        let mut repeat_card = TweakDropdownCard::new(
            "keyboard_repeat",
            "icons/keyboard.svg",
            rust_i18n::t!("tweaks.keyboard_repeat_title").to_string(),
            rust_i18n::t!("tweaks.keyboard_repeat_desc").to_string(),
            dropdown,
        );
        if let Some(ref tooltip_handler) = on_hover_tt_kr {
            let tooltip_handler = tooltip_handler.clone();
            repeat_card = repeat_card.on_hover_tooltip(move |tooltip, window, cx| {
                tooltip_handler(tooltip, window, cx);
            });
        }
        tweak_items.push(repeat_card.into_any_element());

        let ctf_dropdown_name = "ctf_optimization";
        let current_ctf = self
            .pending_selection
            .and_then(|(name, value)| {
                (name == ctf_dropdown_name)
                    .then(|| CtfOptimizationPreset::from_id(value))
                    .flatten()
            })
            .unwrap_or_else(current_ctf_preset);
        let current_ctf_label = ctf_preset_label(current_ctf);
        let hovered_ctf_option = self
            .hovered_option
            .and_then(|(name, option)| (name == ctf_dropdown_name).then_some(option));
        let ctf_options = CtfOptimizationPreset::ALL
            .into_iter()
            .map(|preset| {
                (
                    preset.id(),
                    ctf_preset_label(preset).into(),
                    Some(ctf_preset_icon(preset)),
                )
            })
            .collect();

        let ctf_dropdown = Dropdown::new(
            "ctf_optimization_select",
            current_ctf_label,
            current_ctf.id(),
        )
        .icon(ctf_preset_icon(current_ctf))
        .localized_options(ctf_options)
        .open(self.open_dropdown == Some(ctf_dropdown_name))
        .opening(self.opening_dropdown == Some(ctf_dropdown_name))
        .closing(self.closing_dropdown == Some(ctf_dropdown_name))
        .upward(self.open_dropdown_upward)
        .morphing(self.pending_selection.map(|(name, _)| name) == Some(ctf_dropdown_name))
        .hovered(self.hovered_dropdown == Some(ctf_dropdown_name))
        .hovered_option(hovered_ctf_option)
        .on_toggle(move |window, cx| {
            if let Some(ref handler) = toggle_dropdown_ctf {
                handler(ctf_dropdown_name, window, cx);
            }
        })
        .on_select(move |value, window, cx| {
            if let Some(ref handler) = select_ctf {
                handler(value, window, cx);
            }
        })
        .on_hover_trigger(move |hovered, window, cx| {
            if let Some(ref handler) = hover_dropdown_ctf {
                handler(ctf_dropdown_name, hovered, window, cx);
            }
        })
        .on_hover_option(move |option, hovered, window, cx| {
            if let Some(ref handler) = hover_option_ctf {
                let option = CtfOptimizationPreset::from_id(option)
                    .unwrap_or(CtfOptimizationPreset::Standard)
                    .id();
                handler(ctf_dropdown_name, option, hovered, window, cx);
            }
        })
        .on_close(move |window, cx| {
            if let Some(ref handler) = close_dropdowns_ctf {
                handler(window, cx);
            }
        });

        let mut ctf_badges = Vec::new();
        match current_ctf {
            CtfOptimizationPreset::Aggressive => {
                ctf_badges.push(
                    TweakBadge::new(rust_i18n::t!("tweaks.badge_side_effect_high").to_string())
                        .icon("icons/shield-alert.svg")
                        .tooltip(rust_i18n::t!("tweaks.ctf_side_effect_aggressive").to_string())
                        .color(theme.accent_red),
                );
            }
            CtfOptimizationPreset::Mild => {
                ctf_badges.push(
                    TweakBadge::new(rust_i18n::t!("tweaks.badge_side_effect_low").to_string())
                        .icon("icons/shield-alert.svg")
                        .tooltip(rust_i18n::t!("tweaks.ctf_side_effect_mild").to_string())
                        .color(theme.accent_yellow),
                );
            }
            CtfOptimizationPreset::Standard => {}
        }

        let mut ctf_card = TweakDropdownCard::new(
            "ctf_optimization",
            "icons/type.svg",
            rust_i18n::t!("tweaks.ctf_optimization_title").to_string(),
            rust_i18n::t!("tweaks.ctf_optimization_desc").to_string(),
            ctf_dropdown,
        )
        .badges(ctf_badges);
        if let Some(ref tooltip_handler) = on_hover_tt_ctf {
            let tooltip_handler = tooltip_handler.clone();
            ctf_card = ctf_card.on_hover_tooltip(move |tooltip, window, cx| {
                tooltip_handler(tooltip, window, cx);
            });
        }
        tweak_items.push(ctf_card.into_any_element());

        let snapkey_dropdown_name = "snapkey";
        let current_snapkey = self
            .pending_selection
            .and_then(|(name, value)| {
                (name == snapkey_dropdown_name)
                    .then(|| SnapKeyPreset::from_id(value))
                    .flatten()
            })
            .unwrap_or_else(current_snapkey_preset);
        let current_snapkey_label = snapkey_preset_label(current_snapkey);
        let hovered_snapkey_option = self
            .hovered_option
            .and_then(|(name, option)| (name == snapkey_dropdown_name).then_some(option));
        let snapkey_options = SnapKeyPreset::ALL
            .into_iter()
            .map(|preset| {
                (
                    preset.id(),
                    snapkey_preset_label(preset).into(),
                    Some(snapkey_preset_icon(preset)),
                )
            })
            .collect();

        let snapkey_dropdown = Dropdown::new(
            "snapkey_select",
            current_snapkey_label,
            current_snapkey.id(),
        )
        .icon(snapkey_preset_icon(current_snapkey))
        .localized_options(snapkey_options)
        .open(self.open_dropdown == Some(snapkey_dropdown_name))
        .opening(self.opening_dropdown == Some(snapkey_dropdown_name))
        .closing(self.closing_dropdown == Some(snapkey_dropdown_name))
        .upward(self.open_dropdown_upward)
        .morphing(self.pending_selection.map(|(name, _)| name) == Some(snapkey_dropdown_name))
        .hovered(self.hovered_dropdown == Some(snapkey_dropdown_name))
        .hovered_option(hovered_snapkey_option)
        .on_toggle(move |window, cx| {
            if let Some(ref handler) = toggle_dropdown_sk {
                handler(snapkey_dropdown_name, window, cx);
            }
        })
        .on_select(move |value, window, cx| {
            if let Some(ref handler) = select_snapkey {
                handler(value, window, cx);
            }
        })
        .on_hover_trigger(move |hovered, window, cx| {
            if let Some(ref handler) = hover_dropdown_sk {
                handler(snapkey_dropdown_name, hovered, window, cx);
            }
        })
        .on_hover_option(move |option, hovered, window, cx| {
            if let Some(ref handler) = hover_option_sk {
                let option = SnapKeyPreset::from_id(option)
                    .unwrap_or(SnapKeyPreset::Off)
                    .id();
                handler(snapkey_dropdown_name, option, hovered, window, cx);
            }
        })
        .on_close(move |window, cx| {
            if let Some(ref handler) = close_dropdowns_sk {
                handler(window, cx);
            }
        });

        let mut snapkey_badges = vec![
            TweakBadge::new(rust_i18n::t!("tweaks.snapkey_badge_gaming").to_string())
                .icon("icons/gamepad-2.svg")
                .color(theme.accent_purple),
            TweakBadge::new(rust_i18n::t!("tweaks.badge_side_effect_high").to_string())
                .icon("icons/shield-alert.svg")
                .tooltip(rust_i18n::t!("tweaks.snapkey_side_effect").to_string())
                .color(theme.accent_red),
        ];
        if current_snapkey != SnapKeyPreset::Off {
            snapkey_badges.push(
                TweakBadge::new(rust_i18n::t!("tweaks.snapkey_badge_background").to_string())
                    .icon("icons/activity.svg")
                    .color(theme.accent_blue),
            );
        }

        let mut snapkey_card = TweakDropdownCard::new(
            "snapkey",
            "icons/crosshair-2.svg",
            rust_i18n::t!("tweaks.snapkey_title").to_string(),
            rust_i18n::t!("tweaks.snapkey_desc").to_string(),
            snapkey_dropdown,
        )
        .badges(snapkey_badges);
        if let Some(ref tooltip_handler) = on_hover_tt_sk {
            let tooltip_handler = tooltip_handler.clone();
            snapkey_card = snapkey_card.on_hover_tooltip(move |tooltip, window, cx| {
                tooltip_handler(tooltip, window, cx);
            });
        }
        tweak_items.push(snapkey_card.into_any_element());

        render_tweak_page(route, tweak_items)
    }
}

fn keyboard_repeat_preset_label(preset: KeyboardRepeatPreset) -> String {
    let key = match preset {
        KeyboardRepeatPreset::Standard => "tweaks.keyboard_repeat_standard",
        KeyboardRepeatPreset::Balanced => "tweaks.keyboard_repeat_balanced",
        KeyboardRepeatPreset::Fast => "tweaks.keyboard_repeat_fast",
        KeyboardRepeatPreset::Ultra => "tweaks.keyboard_repeat_ultra",
    };
    rust_i18n::t!(key).to_string()
}

const fn keyboard_repeat_preset_icon(preset: KeyboardRepeatPreset) -> &'static str {
    match preset {
        KeyboardRepeatPreset::Standard => "icons/keyboard.svg",
        KeyboardRepeatPreset::Balanced => "icons/gauge.svg",
        KeyboardRepeatPreset::Fast => "icons/rabbit.svg",
        KeyboardRepeatPreset::Ultra => "icons/zap.svg",
    }
}

fn ctf_preset_label(preset: CtfOptimizationPreset) -> String {
    let key = match preset {
        CtfOptimizationPreset::Standard => "tweaks.ctf_preset_standard",
        CtfOptimizationPreset::Mild => "tweaks.ctf_preset_mild",
        CtfOptimizationPreset::Aggressive => "tweaks.ctf_preset_aggressive",
    };
    rust_i18n::t!(key).to_string()
}

const fn ctf_preset_icon(preset: CtfOptimizationPreset) -> &'static str {
    match preset {
        CtfOptimizationPreset::Standard => "icons/shield-check.svg",
        CtfOptimizationPreset::Mild => "icons/feather.svg",
        CtfOptimizationPreset::Aggressive => "icons/flame.svg",
    }
}
