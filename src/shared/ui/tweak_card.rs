use std::sync::Arc;

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Rgba,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::{Icon, Switch, TooltipState};

pub type TweakCardToggleHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq)]
pub struct TweakBadge {
    pub label: SharedString,
    pub icon: Option<&'static str>,
    pub tooltip: Option<SharedString>,
    pub color: Option<Rgba>,
}

impl TweakBadge {
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            tooltip: None,
            color: None,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    #[must_use]
    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(IntoElement)]
pub struct TweakCard {
    id: &'static str,
    icon: &'static str,
    title: SharedString,
    description: SharedString,
    badges: Vec<TweakBadge>,
    is_applied: bool,
    on_toggle: Option<TweakCardToggleHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl TweakCard {
    #[must_use]
    pub fn new(
        id: &'static str,
        icon: &'static str,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        is_applied: bool,
    ) -> Self {
        Self {
            id,
            icon,
            title: title.into(),
            description: description.into(),
            badges: Vec::new(),
            is_applied,
            on_toggle: None,
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
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
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

impl RenderOnce for TweakCard {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id_str = self.id;
        let is_applied = self.is_applied;
        let on_toggle = self.on_toggle;
        let badges = self.badges;
        let on_hover_tt = self.on_hover_tooltip;

        let icon_box = div()
            .size(px(32.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.input_border)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .child(Icon::new(self.icon).size(px(16.0)).color(theme.accent_blue));

        let switch_id: &'static str = Box::leak(format!("switch_{id_str}").into_boxed_str());
        let switch_el = Switch::new(switch_id, is_applied).on_toggle(move |new_val, window, cx| {
            if let Some(ref h) = on_toggle {
                h(new_val, window, cx);
            }
        });

        // 1. First row: Icon - Title - Action (Switch)
        let header_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .flex_1()
                    .min_w(px(0.0))
                    .child(icon_box)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(13.5))
                            .line_height(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary)
                            .child(self.title),
                    ),
            )
            .child(div().flex_none().child(switch_el));

        // 2. Second row: Description (tightly bound under header)
        let desc_row = div()
            .w_full()
            .text_size(px(11.5))
            .line_height(px(15.0))
            .font_weight(FontWeight::NORMAL)
            .text_color(theme.text_muted)
            .child(self.description);

        let top_content = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .w_full()
            .child(header_row)
            .child(desc_row);

        let mut card = div()
            .id(ElementId::Name(id_str.into()))
            .flex()
            .flex_col()
            .justify_between()
            .w_full()
            .h_full()
            .p(px(12.0))
            .gap(px(6.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(theme.card_border)
            .bg(theme.card_bg)
            .hover(|s| {
                s.border_color(theme.accent_blue)
                    .bg(theme.input_bg.opacity(0.3))
            })
            .child(top_content);

        // 3. Third row: Badges (anchored at bottom)
        if !badges.is_empty() {
            let mut badges_row = div()
                .flex()
                .items_center()
                .flex_wrap()
                .gap(px(6.0))
                .w_full()
                .mt_auto();
            for (i, b) in badges.into_iter().enumerate() {
                let badge_id: &'static str =
                    Box::leak(format!("{id_str}_badge_{i}").into_boxed_str());
                let tt_text = b.tooltip.clone();
                let tt_move = b.tooltip.clone();
                let on_tt_hov = on_hover_tt.clone();
                let on_tt_move = on_hover_tt.clone();

                let mut badge_pill = div()
                    .id(ElementId::Name(badge_id.into()))
                    .flex()
                    .items_center()
                    .h(px(20.0))
                    .px(px(6.5))
                    .gap(px(4.5))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(theme.input_border)
                    .bg(theme.input_bg);

                if let Some(ref tt) = tt_text {
                    let tt_captured = tt.clone();
                    let tt_move_captured = tt_move.clone();
                    badge_pill = badge_pill
                        .cursor_pointer()
                        .on_hover(move |&hov, window, cx| {
                            if let Some(ref h) = on_tt_hov {
                                if hov {
                                    let pos = window.mouse_position();
                                    h(
                                        Some(TooltipState {
                                            text: tt_captured.clone(),
                                            cursor_pos: pos,
                                        }),
                                        window,
                                        cx,
                                    );
                                } else {
                                    h(None, window, cx);
                                }
                            }
                        })
                        .on_mouse_move(move |_, window, cx| {
                            if let Some(ref h) = on_tt_move {
                                if let Some(ref txt) = tt_move_captured {
                                    let pos = window.mouse_position();
                                    h(
                                        Some(TooltipState {
                                            text: txt.clone(),
                                            cursor_pos: pos,
                                        }),
                                        window,
                                        cx,
                                    );
                                }
                            }
                        });
                }

                if let Some(icon_path) = b.icon {
                    let icon_color = b.color.unwrap_or(theme.text_muted);
                    badge_pill =
                        badge_pill.child(Icon::new(icon_path).size(px(11.0)).color(icon_color));
                }

                let text_color = b.color.unwrap_or(theme.text_muted);
                badge_pill = badge_pill.child(
                    div()
                        .text_size(px(10.5))
                        .line_height(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_color)
                        .child(b.label),
                );

                badges_row = badges_row.child(badge_pill);
            }
            card = card.child(badges_row);
        }

        card
    }
}
