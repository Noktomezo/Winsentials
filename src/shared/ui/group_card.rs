use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString,
    Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

#[derive(IntoElement)]
pub struct GroupCard {
    icon: SharedString,
    title: SharedString,
    description: SharedString,
    icon_color: Option<Rgba>,
    header_action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl GroupCard {
    #[must_use]
    pub fn new(
        icon: impl Into<SharedString>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            description: description.into(),
            icon_color: None,
            header_action: None,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn icon_color(mut self, color: Rgba) -> Self {
        self.icon_color = Some(color);
        self
    }

    #[must_use]
    pub fn header_action(mut self, action: impl IntoElement) -> Self {
        self.header_action = Some(action.into_any_element());
        self
    }

    #[must_use]
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for GroupCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let icon_col = self.icon_color.unwrap_or(theme.accent_cyan);

        // Header icon box: exactly 32x32px matching button dimensions and 16px icon
        let icon_box = div()
            .size(px(32.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.card_border)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .child(Icon::new(self.icon).size(px(16.0)).color(icon_col));

        // Text stack: aligned to exactly 32px height matching the icon box
        let text_stack = div()
            .flex()
            .flex_col()
            .justify_between()
            .h(px(32.0))
            .child(
                div()
                    .text_size(px(13.5))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_primary)
                    .child(self.title),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(self.description),
            );

        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(32.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(icon_box)
                    .child(text_stack),
            );

        if let Some(action) = self.header_action {
            header = header.child(action);
        }

        let divider = div().h(px(1.0)).bg(theme.card_border).w_full();

        let body = div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .children(self.children);

        // Card container uses Frost Slate bg-2 (theme.card_bg) + border (theme.card_border)
        // No overflow_hidden so that child dropdowns render over without getting clipped!
        div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(10.0))
            .border_1()
            .border_color(theme.card_border)
            .bg(theme.card_bg)
            .p(px(16.0))
            .gap(px(16.0))
            .child(header)
            .child(divider)
            .child(body)
    }
}
