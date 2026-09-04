use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px,
};

use crate::shared::theme::Theme;

#[derive(IntoElement)]
pub struct PageHeader {
    title: SharedString,
    description: SharedString,
    badge: Option<AnyElement>,
    actions: Option<AnyElement>,
}

impl PageHeader {
    #[must_use]
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            badge: None,
            actions: None,
        }
    }

    #[must_use]
    pub fn badge(mut self, badge: impl IntoElement) -> Self {
        self.badge = Some(badge.into_any_element());
        self
    }

    #[must_use]
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

impl RenderOnce for PageHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let badge = self.badge.map(|badge| div().flex_none().child(badge));
        let actions = self.actions.map(|actions| div().flex_none().child(actions));

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.0))
            .w_full()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .min_w(px(0.0))
                                    .text_size(px(20.0))
                                    .line_height(px(24.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .text_ellipsis()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(self.title),
                            )
                            .children(badge),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .line_height(px(16.0))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_muted)
                            .text_ellipsis()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(self.description),
                    ),
            )
            .children(actions)
    }
}
