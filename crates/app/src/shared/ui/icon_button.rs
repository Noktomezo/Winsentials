use std::sync::Arc;

use gpui::{
    App, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type ClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
#[allow(dead_code)]
pub struct IconButton {
    id: ElementId,
    icon_path: SharedString,
    icon_size: Pixels,
    icon_color: Option<Hsla>,
    selected: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

#[allow(dead_code)]
impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon_path: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon_path: icon_path.into(),
            icon_size: px(16.0),
            icon_color: None,
            selected: false,
            tooltip: None,
            on_click: None,
        }
    }

    #[must_use]
    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
        self
    }

    #[must_use]
    pub fn icon_color(mut self, color: impl Into<Hsla>) -> Self {
        self.icon_color = Some(color.into());
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let color = self.icon_color.unwrap_or_else(|| {
            if self.selected {
                theme.accent_cyan.into()
            } else {
                theme.text_primary.into()
            }
        });

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(px(32.0))
            .rounded(px(6.0))
            .cursor_pointer();

        if self.selected {
            base = base.bg(theme.accent_selected_bg);
        } else {
            base = base
                .hover(move |s| s.bg(theme.accent_hover_bg).text_color(theme.accent_cyan))
                .active(move |s| s.bg(theme.accent_active_bg));
        }

        if let Some(on_click) = self.on_click {
            base = base.on_click(move |event, window, cx| {
                (on_click)(event, window, cx);
            });
        }

        let icon = Icon::new(self.icon_path).size(self.icon_size).color(color);

        base.child(icon)
    }
}
