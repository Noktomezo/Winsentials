use std::path::Path;
use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, ParentElement, Rgba, SharedString, Styled, Window, div, img, px};

use crate::components::icon::Icon;

pub type DropdownSelectHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownDeleteHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownCloseHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DropdownOptionHoverHandler = Arc<dyn Fn(&str, &bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownItem {
    pub value: &'static str,
    pub label: SharedString,
    pub icon: Option<&'static str>,
    pub deletable: bool,
}

impl DropdownItem {
    #[must_use]
    pub fn new(
        value: &'static str,
        label: impl Into<SharedString>,
        icon: Option<&'static str>,
    ) -> Self {
        Self {
            value,
            label: label.into(),
            icon,
            deletable: false,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn deletable(mut self, deletable: bool) -> Self {
        self.deletable = deletable;
        self
    }
}

#[must_use]
pub fn render_dropdown_icon(icon_path: &str, current_color: Rgba) -> AnyElement {
    if Path::new(icon_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
    {
        div()
            .flex_none()
            .child(
                img(icon_path.to_string())
                    .w(px(16.0))
                    .h(px(11.0))
                    .rounded(px(2.0)),
            )
            .into_any_element()
    } else {
        div()
            .flex_none()
            .child(
                Icon::new(icon_path.to_string())
                    .size(px(14.0))
                    .color(current_color),
            )
            .into_any_element()
    }
}