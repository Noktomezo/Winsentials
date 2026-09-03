use std::sync::Arc;

use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

use crate::shared::theme::Theme;
use crate::shared::ui::dropdown::Dropdown;
use crate::shared::ui::tooltip::TooltipState;
use crate::shared::ui::tweak_card::{TooltipHoverHandler, TweakBadge, render_tweak_card_shell};

#[derive(IntoElement)]
#[allow(dead_code)]
pub struct TweakDropdownCard {
    id: &'static str,
    icon: &'static str,
    title: SharedString,
    description: SharedString,
    dropdown: Dropdown,
    badges: Vec<TweakBadge>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

#[allow(dead_code)]
impl TweakDropdownCard {
    #[must_use]
    pub fn new(
        id: &'static str,
        icon: &'static str,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        dropdown: Dropdown,
    ) -> Self {
        Self {
            id,
            icon,
            title: title.into(),
            description: description.into(),
            dropdown,
            badges: Vec::new(),
            on_hover_tooltip: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn badge(mut self, badge: TweakBadge) -> Self {
        self.badges.push(badge);
        self
    }

    #[must_use]
    pub fn badges(mut self, badges: Vec<TweakBadge>) -> Self {
        self.badges = badges;
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

impl RenderOnce for TweakDropdownCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id_str = self.id;
        let hover_state = window.use_keyed_state((id_str, 2usize), cx, |_, _| false);
        let hovered = *hover_state.read(cx);

        render_tweak_card_shell(
            id_str,
            self.icon,
            self.title,
            self.description,
            self.badges,
            self.on_hover_tooltip.as_ref(),
            self.dropdown,
            hover_state,
            hovered,
            &theme,
            cx.reduce_motion(),
        )
    }
}
