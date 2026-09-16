use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div, px,
};

use super::{SmoothScrollState, render_scroll_viewport};

pub type VirtualItemRenderer = Arc<dyn Fn(usize, &mut Window, &mut App) -> AnyElement>;

#[derive(IntoElement)]
pub struct SmoothVirtualList {
    id: &'static str,
    header: Option<AnyElement>,
    total_items: usize,
    item_height: Pixels,
    gap: Pixels,
    render_item: VirtualItemRenderer,
}

impl SmoothVirtualList {
    #[must_use]
    pub fn new(
        id: &'static str,
        total_items: usize,
        item_height: Pixels,
        gap: Pixels,
        render_item: impl Fn(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id,
            header: None,
            total_items,
            item_height,
            gap,
            render_item: Arc::new(render_item),
        }
    }

    #[must_use]
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }
}

impl RenderOnce for SmoothVirtualList {
    #[allow(clippy::too_many_lines)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state((self.id, 0usize), cx, |_, _| SmoothScrollState::default());
        let handle = state.read(cx).handle.clone();

        let offset_y = (-handle.offset().y).max(px(0.0));
        let viewport_h = if handle.bounds().size.height > px(0.0) {
            handle.bounds().size.height
        } else {
            window.viewport_size().height
        };

        let total_items = self.total_items;
        let item_h = self.item_height;
        let gap = self.gap;
        let stride = item_h + gap;

        let items_content = if total_items == 0 {
            div().into_any_element()
        } else {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let first_visible = ((offset_y / stride).floor() as usize).min(total_items);
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let visible_count = ((viewport_h / stride).ceil() as usize).max(1) + 2;

            let overscan = 6usize;
            let start_idx = first_visible.saturating_sub(overscan);
            let end_idx = (first_visible + visible_count + overscan).min(total_items);

            #[allow(clippy::cast_precision_loss)]
            let top_spacer = stride * start_idx as f32;
            #[allow(clippy::cast_precision_loss)]
            let bottom_spacer = stride * (total_items.saturating_sub(end_idx)) as f32;

            let mut visible_elements = Vec::with_capacity(end_idx - start_idx);
            for i in start_idx..end_idx {
                visible_elements.push((self.render_item)(i, window, cx));
            }

            div()
                .flex()
                .flex_col()
                .w_full()
                .when(top_spacer > px(0.0), |this| {
                    this.child(div().h(top_spacer).w_full().flex_none())
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(gap)
                        .w_full()
                        .children(visible_elements),
                )
                .when(bottom_spacer > px(0.0), |this| {
                    this.child(div().h(bottom_spacer).w_full().flex_none())
                })
                .into_any_element()
        };

        let content = div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .children(self.header)
            .child(items_content)
            .into_any_element();

        render_scroll_viewport(self.id, content, window, cx)
    }
}
