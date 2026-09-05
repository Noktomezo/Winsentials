use std::sync::Arc;

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;

pub type StringHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type BoolHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type OptionHoverHandler =
    Arc<dyn Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static>;
pub type VoidHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;

pub(crate) fn settings_row_text(
    title: impl Into<SharedString>,
    desc: impl Into<SharedString>,
    theme: &Theme,
) -> Div {
    div()
        .flex()
        .flex_col()
        .justify_between()
        .h(px(32.0))
        .child(
            div()
                .text_size(px(13.5))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(11.5))
                .line_height(px(14.0))
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_muted)
                .child(desc.into()),
        )
}

pub(crate) fn settings_row(left: impl IntoElement, right: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .child(left)
        .child(right)
}