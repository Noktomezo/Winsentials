use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;

#[derive(IntoElement, Default)]
pub struct FeaturesPage;

impl RenderOnce for FeaturesPage {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let route = AppRoute::Features;

        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
    }
}
