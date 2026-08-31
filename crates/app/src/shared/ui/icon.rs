use gpui::{
    App, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div,
    px, svg,
};

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    size: Pixels,
    color: Option<Hsla>,
}

impl Icon {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            size: px(16.0),
            color: None,
        }
    }

    #[must_use]
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut icon_svg = svg().path(self.path).size(self.size).flex_none();

        if let Some(color) = self.color {
            icon_svg = icon_svg.text_color(color);
        }

        div()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size)
            .flex_none()
            .child(icon_svg)
    }
}
