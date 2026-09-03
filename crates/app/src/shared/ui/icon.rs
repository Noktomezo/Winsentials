use gpui::{
    App, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div,
    img, px, svg,
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
        let is_raster = self.path.ends_with(".png");

        let icon_el = if is_raster {
            img(self.path)
                .size(self.size)
                .flex_none()
                .into_any_element()
        } else {
            let mut icon_svg = svg().path(self.path).size(self.size).flex_none();
            if let Some(color) = self.color {
                icon_svg = icon_svg.text_color(color);
            }
            icon_svg.into_any_element()
        };

        div()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size)
            .flex_none()
            .child(icon_el)
    }
}
