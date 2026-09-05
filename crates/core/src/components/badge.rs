use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};

use crate::components::icon::Icon;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Outline,
    Neutral,
    Ghost,
    Secondary,
    Success,
    Accent,
    Warning,
    Destructive,
    Muted,
}

#[derive(IntoElement)]
pub struct Badge {
    id: ElementId,
    label: SharedString,
    variant: BadgeVariant,
    icon: Option<&'static str>,
}

impl Badge {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: BadgeVariant::Outline,
            icon: None,
        }
    }

    #[must_use]
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);

        let (bg, border, text_color) = match self.variant {
            BadgeVariant::Outline | BadgeVariant::Neutral => {
                (theme.input_bg, Some(theme.card_border), theme.text_primary)
            }
            BadgeVariant::Ghost => (gpui::rgba(0x0000_0000), None, theme.text_muted),
            BadgeVariant::Secondary | BadgeVariant::Muted => (
                theme.input_bg.opacity(0.5),
                Some(theme.card_border.opacity(0.6)),
                theme.text_muted,
            ),
            BadgeVariant::Success => (
                theme.accent_green.opacity(0.14),
                Some(theme.accent_green.opacity(0.35)),
                theme.accent_green,
            ),
            BadgeVariant::Accent => (
                theme.accent_blue.opacity(0.14),
                Some(theme.accent_blue.opacity(0.35)),
                theme.accent_blue,
            ),
            BadgeVariant::Warning => (
                theme.accent_yellow.opacity(0.14),
                Some(theme.accent_yellow.opacity(0.35)),
                theme.accent_yellow,
            ),
            BadgeVariant::Destructive => (
                theme.accent_red.opacity(0.14),
                Some(theme.accent_red.opacity(0.35)),
                theme.accent_red,
            ),
        };

        let icon_el = self
            .icon
            .map(|p| Icon::new(p).size(px(11.0)).color(text_color));

        let id_str = format!("{:?}", self.id);

        let mut el = div()
            .id(self.id)
            .debug_selector(move || id_str.clone())
            .flex()
            .items_center()
            .justify_center()
            .gap(px(4.0))
            .h(px(20.0))
            .px(px(6.0))
            .rounded(px(4.0))
            .bg(bg)
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color);

        if let Some(border_col) = border {
            el = el.border_1().border_color(border_col);
        }

        el.children(icon_el).child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, VisualTestContext, size};

    struct TestBadgeView;

    impl Render for TestBadgeView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(Badge::new("b1", "v0.9.0").variant(BadgeVariant::Neutral))
                .child(Badge::new("b2", "Последняя").variant(BadgeVariant::Success))
                .child(Badge::new("b3", "Есть новее").variant(BadgeVariant::Accent))
        }
    }

    #[gpui::test]
    fn badge_renders_without_panic(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(400.0), px(200.0)), |_, _| TestBadgeView);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let bounds = cx.debug_bounds("Name(\"b1\")");
        assert!(bounds.is_some());
    }
}
