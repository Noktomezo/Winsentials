use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::entities::tweaks::{TweakCategory, get_all_tweaks};
use crate::features::navigation::AppRoute;
use crate::pages::context_menu_page::build_tweak_badges;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TweakCard;
use crate::shared::ui::animated_grid::render_animated_grid;
use crate::shared::ui::tweak_card::TooltipHoverHandler;

pub type TweakToggleHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct InputPage {
    windows_build: u32,
    sidebar_expanded: bool,
    on_toggle_tweak: Option<TweakToggleHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl InputPage {
    #[must_use]
    pub fn new(windows_build: u32, sidebar_expanded: bool) -> Self {
        Self {
            windows_build,
            sidebar_expanded,
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

impl RenderOnce for InputPage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Input;
        let windows_build = self.windows_build;
        let on_toggle = self.on_toggle_tweak;
        let on_hover_tt = self.on_hover_tooltip;

        let all_tweaks = get_all_tweaks();
        let mut tweak_items: Vec<(&'static str, AnyElement)> = Vec::new();

        for tweak in all_tweaks {
            if tweak.category == TweakCategory::Input && tweak.is_supported(windows_build) {
                let is_applied = (tweak.is_applied)();
                let tweak_id = tweak.id;
                let toggle_cb = on_toggle.clone();
                let badges = build_tweak_badges(tweak, &theme);

                let mut card = TweakCard::new(
                    tweak.id,
                    tweak.icon,
                    rust_i18n::t!(tweak.title_key).to_string(),
                    rust_i18n::t!(tweak.desc_key).to_string(),
                    is_applied,
                )
                .badges(badges)
                .on_toggle(move |new_val, window, cx| {
                    if let Some(ref h) = toggle_cb {
                        h(tweak_id, new_val, window, cx);
                    }
                });

                if let Some(ref tt_fn) = on_hover_tt {
                    let tt_c = tt_fn.clone();
                    card = card.on_hover_tooltip(move |tt, window, cx| {
                        tt_c(tt, window, cx);
                    });
                }

                tweak_items.push((tweak.id, card.into_any_element()));
            }
        }

        let sidebar_w = if self.sidebar_expanded {
            px(200.0)
        } else {
            px(40.0)
        };
        let available_width = (window.viewport_size().width - sidebar_w - px(32.0)).max(px(320.0));
        let tweak_grid = render_animated_grid(
            "input_tweaks_grid",
            available_width,
            px(360.0),
            px(112.0),
            px(12.0),
            tweak_items,
        );

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
            .child(tweak_grid)
    }
}
