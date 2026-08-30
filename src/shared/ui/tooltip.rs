use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Window, div, ease_in_out, px,
};

use crate::shared::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipState {
    pub text: SharedString,
    pub cursor_pos: Point<Pixels>,
}

#[derive(IntoElement)]
pub struct Tooltip {
    text: SharedString,
    cursor_pos: Point<Pixels>,
}

impl Tooltip {
    #[must_use]
    pub fn new(text: impl Into<SharedString>, cursor_pos: Point<Pixels>) -> Self {
        Self {
            text: text.into(),
            cursor_pos,
        }
    }
}

impl RenderOnce for Tooltip {
    #[allow(clippy::cast_precision_loss)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let viewport = window.viewport_size();

        // Estimated dimensions for boundary calculation
        let text_chars = self.text.chars().count() as f32;
        let est_width = px((text_chars * 7.5 + 20.0).max(40.0));
        let est_height = px(26.0);

        let cursor_x = self.cursor_pos.x;
        let cursor_y = self.cursor_pos.y;

        // Default position: top-right of cursor
        // X calculation: place right of cursor (+12px), flip left (-12px - width) if exceeding window width
        let pos_x = if cursor_x + px(12.0) + est_width > viewport.width - px(8.0) {
            cursor_x - est_width - px(12.0)
        } else {
            cursor_x + px(12.0)
        };

        // Y calculation: place above cursor (-30px), flip below (+18px) if hitting top of window / titlebar
        let pos_y = if cursor_y - est_height - px(6.0) < px(36.0) {
            cursor_y + px(18.0)
        } else {
            cursor_y - est_height - px(6.0)
        };

        // Safe clamping within window viewport
        let safe_x = pos_x.max(px(8.0)).min(viewport.width - est_width - px(8.0));
        let safe_y = pos_y
            .max(px(36.0))
            .min(viewport.height - est_height - px(8.0));

        div()
            .id(ElementId::Name("global_cursor_tooltip".into()))
            .absolute()
            .top(safe_y)
            .left(safe_x)
            .flex()
            .items_center()
            .justify_center()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.input_border)
            .bg(theme.input_bg)
            .shadow_md()
            .with_animation(
                ElementId::Name("tooltip_enter".into()),
                Animation::new(Duration::from_millis(120)).with_easing(ease_in_out),
                move |box_el, delta| {
                    let opacity = delta;
                    let offset_y = (1.0 - delta) * 3.0;
                    box_el.opacity(opacity).mt(px(offset_y))
                },
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(15.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_primary)
                    .child(self.text),
            )
    }
}
