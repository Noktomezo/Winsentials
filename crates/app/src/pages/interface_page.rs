use std::sync::Arc;

use gpui::{App, IntoElement, RenderOnce, Window};

use crate::entities::tweaks::TweakCategory;
use crate::features::navigation::AppRoute;
use crate::pages::context_menu_page::{render_tweak_cards_for_category, render_tweak_page};
use crate::shared::theme::Theme;
use crate::shared::ui::tweak_card::TooltipHoverHandler;

pub type TweakToggleHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct InterfacePage {
    windows_build: u32,
    on_toggle_tweak: Option<TweakToggleHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl InterfacePage {
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

impl RenderOnce for InterfacePage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let tweak_items = render_tweak_cards_for_category(
            TweakCategory::Interface,
            self.windows_build,
            &theme,
            self.on_toggle_tweak.as_ref(),
            self.on_hover_tooltip.as_ref(),
            cx,
        );
        render_tweak_page(AppRoute::Interface, tweak_items)
    }
}
