use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::entities::tweaks::{
    RestartRequirement, SideEffectLevel, TweakCategory, TweakDefinition, get_all_tweaks,
};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TweakCard;
use crate::shared::ui::tweak_card::{TooltipHoverHandler, TweakBadge};

pub type TweakToggleHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct ContextMenuPage {
    windows_build: u32,
    on_toggle_tweak: Option<TweakToggleHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl ContextMenuPage {
    #[must_use]
    pub fn new(windows_build: u32) -> Self {
        Self {
            windows_build,
            on_toggle_tweak: None,
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
    pub fn on_hover_tooltip(
        mut self,
        handler: impl Fn(Option<crate::shared::ui::TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }
}

pub(crate) fn build_tweak_badges(tweak: &TweakDefinition, theme: &Theme) -> Vec<TweakBadge> {
    let mut badges = Vec::new();
    match tweak.restart {
        RestartRequirement::Explorer => {
            badges.push(
                TweakBadge::new(rust_i18n::t!("tweaks.badge_restart_explorer").to_string())
                    .icon("icons/rotate-ccw.svg")
                    .tooltip(rust_i18n::t!("tweaks.badge_restart_explorer_tooltip").to_string())
                    .color(theme.accent_blue),
            );
        }
        RestartRequirement::Logoff => {
            badges.push(
                TweakBadge::new(rust_i18n::t!("tweaks.badge_logoff").to_string())
                    .icon("icons/log-out.svg")
                    .tooltip(rust_i18n::t!("tweaks.badge_logoff_tooltip").to_string())
                    .color(theme.accent_blue),
            );
        }
        RestartRequirement::Reboot => {
            badges.push(
                TweakBadge::new(rust_i18n::t!("tweaks.badge_reboot").to_string())
                    .icon("icons/power.svg")
                    .tooltip(rust_i18n::t!("tweaks.badge_reboot_tooltip").to_string())
                    .color(theme.accent_blue),
            );
        }
        RestartRequirement::None => {}
    }
    if tweak.min_build == Some(22000) {
        badges.push(
            TweakBadge::new(rust_i18n::t!("tweaks.badge_win11").to_string())
                .icon("icons/monitor.svg")
                .tooltip(rust_i18n::t!("tweaks.badge_win11_tooltip").to_string())
                .color(theme.accent_cyan),
        );
    }
    if let Some(side_effect) = tweak.side_effect {
        let (label, color) = match side_effect.level {
            SideEffectLevel::Low => (
                rust_i18n::t!("tweaks.badge_side_effect_low").to_string(),
                theme.accent_yellow,
            ),
            SideEffectLevel::Medium => (
                rust_i18n::t!("tweaks.badge_side_effect_medium").to_string(),
                theme.accent_orange,
            ),
            SideEffectLevel::High => (
                rust_i18n::t!("tweaks.badge_side_effect_high").to_string(),
                theme.accent_red,
            ),
        };
        badges.push(
            TweakBadge::new(label)
                .icon("icons/shield-alert.svg")
                .tooltip(rust_i18n::t!(side_effect.description_key).to_string())
                .color(color),
        );
    }
    badges
}

pub(crate) fn render_tweak_cards_for_category(
    category: TweakCategory,
    windows_build: u32,
    theme: &Theme,
    on_toggle: Option<&TweakToggleHandler>,
    on_hover_tt: Option<&TooltipHoverHandler>,
    cx: &App,
) -> Vec<(&'static str, AnyElement)> {
    let all_tweaks = get_all_tweaks();
    let highlighted_tweak_id = cx
        .has_global::<crate::entities::tweaks::HighlightedTweak>()
        .then(|| {
            cx.global::<crate::entities::tweaks::HighlightedTweak>()
                .tweak_id
        })
        .flatten();
    let mut tweak_items: Vec<(&'static str, AnyElement)> = Vec::new();

    for tweak in all_tweaks {
        if tweak.category == category && tweak.is_supported(windows_build) {
            let badges = build_tweak_badges(tweak, theme);
            let is_applied = if cx.has_global::<crate::entities::tweaks::TweakStates>() {
                cx.global::<crate::entities::tweaks::TweakStates>()
                    .is_applied(tweak)
            } else {
                (tweak.is_applied)()
            };
            let tweak_id = tweak.id;
            let is_highlighted = highlighted_tweak_id == Some(tweak.id);
            let toggle_cb = on_toggle.cloned();

            let mut card = TweakCard::new(
                tweak.id,
                tweak.icon,
                rust_i18n::t!(tweak.title_key).to_string(),
                rust_i18n::t!(tweak.desc_key).to_string(),
                is_applied,
            )
            .badges(badges)
            .highlighted(is_highlighted)
            .on_toggle(move |new_val, window, cx| {
                if let Some(ref h) = toggle_cb {
                    h(tweak_id, new_val, window, cx);
                }
            });

            if let Some(tt_fn) = on_hover_tt {
                let tt_c = (*tt_fn).clone();
                card = card.on_hover_tooltip(move |tt, window, cx| {
                    tt_c(tt, window, cx);
                });
            }

            tweak_items.push((tweak.id, card.into_any_element()));
        }
    }

    tweak_items
}

pub(crate) fn render_tweak_page(
    route: AppRoute,
    tweak_items: Vec<(&'static str, AnyElement)>,
    cx: &App,
) -> gpui::Div {
    let mut elements = Vec::with_capacity(tweak_items.len());
    let mut target_index = None;
    let target_tweak = cx
        .try_global::<crate::entities::tweaks::HighlightedTweak>()
        .and_then(|h| if h.needs_scroll { h.tweak_id } else { None });

    for (idx, (tweak_id, element)) in tweak_items.into_iter().enumerate() {
        if target_tweak == Some(tweak_id) {
            target_index = Some(idx);
        }
        elements.push(element);
    }

    let mut tweak_grid = div().grid().grid_cols(1).gap(px(12.0)).children(elements);

    if let Some(idx) = target_index {
        let route_id = route.id();
        tweak_grid = tweak_grid.on_children_prepainted(move |bounds_vec, window, cx| {
            let should_scroll = cx
                .try_global::<crate::entities::tweaks::HighlightedTweak>()
                .is_some_and(|h| h.needs_scroll);
            if !should_scroll {
                return;
            }
            let Some(&card_bounds) = bounds_vec.get(idx) else {
                return;
            };

            let scrolled = crate::shared::ui::SmoothScroll::scroll_bounds_into_view(
                route_id,
                card_bounds,
                px(24.0),
                window,
                cx,
            );

            if scrolled {
                cx.set_global(crate::entities::tweaks::HighlightedTweak {
                    tweak_id: target_tweak,
                    needs_scroll: false,
                });
            } else {
                window.on_next_frame(move |window, cx| {
                    let scrolled = crate::shared::ui::SmoothScroll::scroll_bounds_into_view(
                        route_id,
                        card_bounds,
                        px(24.0),
                        window,
                        cx,
                    );
                    if scrolled {
                        cx.set_global(crate::entities::tweaks::HighlightedTweak {
                            tweak_id: target_tweak,
                            needs_scroll: false,
                        });
                    }
                });
            }
        });
    }

    div()
        .flex()
        .flex_col()
        .w_full()
        .p(px(16.0))
        .gap(px(16.0))
        .child(PageHeader::new(route.title(), route.description()))
        .child(tweak_grid)
}

impl RenderOnce for ContextMenuPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let tweak_items = render_tweak_cards_for_category(
            TweakCategory::ContextMenu,
            self.windows_build,
            &theme,
            self.on_toggle_tweak.as_ref(),
            self.on_hover_tooltip.as_ref(),
            cx,
        );
        render_tweak_page(AppRoute::ContextMenu, tweak_items, cx)
    }
}
