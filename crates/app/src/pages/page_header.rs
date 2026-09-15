use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px,
};

use crate::shared::theme::Theme;

#[derive(IntoElement)]
pub struct PageHeader {
    title: SharedString,
    custom_title: Option<AnyElement>,
    description: SharedString,
    badge: Option<AnyElement>,
    actions: Option<AnyElement>,
}

impl PageHeader {
    #[must_use]
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            custom_title: None,
            description: description.into(),
            badge: None,
            actions: None,
        }
    }

    #[must_use]
    pub fn custom_title(mut self, title: impl IntoElement) -> Self {
        self.custom_title = Some(title.into_any_element());
        self
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
        let title_node = self.custom_title.unwrap_or_else(|| {
            div()
                .min_w(px(0.0))
                .text_size(px(20.0))
                .line_height(px(24.0))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.text_primary)
                .text_ellipsis()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(self.title)
                .into_any_element()
        });

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
                            .child(title_node)
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, InteractiveElement, Render, TestAppContext, size};

    struct TestPageHeaderView;

    impl Render for TestPageHeaderView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            PageHeader::new("Default Title", "Description").custom_title(
                div()
                    .debug_selector(|| "custom_header_title".to_string())
                    .child("CUSTOM"),
            )
        }
    }

    #[gpui::test]
    fn test_page_header_custom_title_renders(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(400.0), px(100.0)), |_window, _cx| {
            TestPageHeaderView
        });

        let mut visual_cx = gpui::VisualTestContext::from_window(window.into(), cx);
        visual_cx.run_until_parked();

        assert!(visual_cx.debug_bounds("custom_header_title").is_some());
    }
}
