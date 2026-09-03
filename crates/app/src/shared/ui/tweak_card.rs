use std::sync::Arc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, Entity, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::{Icon, Switch, TooltipState};
use crate::widgets::sidebar::lerp_rgba;

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

pub(super) fn render_badge_icons(
    id: &'static str,
    badges: Vec<TweakBadge>,
    on_hover_tooltip: Option<&TooltipHoverHandler>,
    theme: &Theme,
) -> AnyElement {
    let mut badges_row = div().flex().items_center().gap(px(4.0)).flex_none();

    for (index, badge) in badges.into_iter().enumerate() {
        let Some(icon) = badge.icon else {
            continue;
        };
        let tooltip: SharedString = badge.tooltip.map_or_else(
            || badge.label.clone(),
            |detail| format!("{}\n{detail}", badge.label).into(),
        );
        let tooltip_move = tooltip.clone();
        let hover_handler = on_hover_tooltip.cloned();
        let move_handler = on_hover_tooltip.cloned();
        let color = badge.color.unwrap_or(theme.text_muted);
        let badge_icon = div()
            .id(ElementId::Name(format!("{id}_badge_{index}").into()))
            .flex()
            .items_center()
            .justify_center()
            .size(px(16.0))
            .cursor_pointer()
            .on_hover(move |&hovered, window, cx| {
                if let Some(ref handler) = hover_handler {
                    if hovered {
                        handler(
                            Some(TooltipState {
                                text: tooltip.clone(),
                                cursor_pos: window.mouse_position(),
                            }),
                            window,
                            cx,
                        );
                    } else {
                        handler(None, window, cx);
                    }
                }
            })
            .on_mouse_move(move |event, window, cx| {
                if let Some(ref handler) = move_handler {
                    handler(
                        Some(TooltipState {
                            text: tooltip_move.clone(),
                            cursor_pos: event.position,
                        }),
                        window,
                        cx,
                    );
                }
            })
            .child(Icon::new(icon).size(px(12.0)).color(color));
        badges_row = badges_row.child(badge_icon);
    }

    badges_row.into_any_element()
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

#[allow(clippy::too_many_arguments)]
pub fn render_tweak_card_shell(
    id: &'static str,
    icon: &'static str,
    title: SharedString,
    description: SharedString,
    badges: Vec<TweakBadge>,
    on_hover_tooltip: Option<&TooltipHoverHandler>,
    action_element: impl IntoElement,
    hover_state: Entity<bool>,
    hovered: bool,
    theme: &Theme,
    reduce_motion: bool,
) -> AnyElement {
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
        .child(Icon::new(icon).size(px(16.0)).color(theme.accent_blue));

    let badge_icons = render_badge_icons(id, badges, on_hover_tooltip, theme);

    // 1. First row: Icon - Title - Action
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
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .flex_shrink_1()
                                .min_w(px(0.0))
                                .text_size(px(13.5))
                                .line_height(px(16.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_primary)
                                .text_ellipsis()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(title),
                        )
                        .child(badge_icons),
                ),
        )
        .child(div().flex_none().child(action_element));

    // 2. Second row: Description (tightly bound under header)
    let desc_row = div()
        .w_full()
        .text_size(px(11.5))
        .line_height(px(15.0))
        .font_weight(FontWeight::NORMAL)
        .text_color(theme.text_muted)
        .child(description);

    let hover_state_for_event = hover_state;
    let card = div()
        .id(ElementId::Name(id.into()))
        .flex()
        .flex_col()
        .w_full()
        .p(px(16.0))
        .gap(px(16.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(theme.card_border)
        .bg(theme.card_bg)
        .on_hover(move |&hovered, _, cx| {
            hover_state_for_event.update(cx, |state, cx| {
                *state = hovered;
                cx.notify();
            });
        })
        .child(header_row)
        .child(desc_row);

    let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
        .to(if hovered { 1.0 } else { 0.0 })
        .with_epsilon(0.01);
    let card_bg = theme.card_bg;
    let hover_bg = theme.input_bg.opacity(0.3);
    let card_border = theme.card_border;
    let hover_border = theme.accent_blue.opacity(0.5);

    if reduce_motion {
        return card
            .bg(if hovered { hover_bg } else { card_bg })
            .border_color(if hovered { hover_border } else { card_border })
            .into_any_element();
    }

    card.with_spring(
        ElementId::Name(format!("{id}_hover_spring").into()),
        spring,
        move |card, value| {
            let progress = value.clamp(0.0, 1.0);
            card.bg(lerp_rgba(card_bg, hover_bg, progress))
                .border_color(lerp_rgba(card_border, hover_border, progress))
        },
    )
    .into_any_element()
}

impl RenderOnce for TweakCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id_str = self.id;
        let is_applied = self.is_applied;
        let on_toggle = self.on_toggle;
        let hover_state = window.use_keyed_state((id_str, 1usize), cx, |_, _| false);
        let hovered = *hover_state.read(cx);

        let switch_el = Switch::new(format!("switch_{id_str}"), is_applied).on_toggle(
            move |new_val, window, cx| {
                if let Some(ref h) = on_toggle {
                    h(new_val, window, cx);
                }
            },
        );

        render_tweak_card_shell(
            id_str,
            self.icon,
            self.title,
            self.description,
            self.badges,
            self.on_hover_tooltip.as_ref(),
            switch_el,
            hover_state,
            hovered,
            &theme,
            cx.reduce_motion(),
        )
    }
}
