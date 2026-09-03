use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Window, div, ease_in_out, px,
};

use crate::shared::theme::Theme;

const TOOLTIP_MAX_WIDTH: f32 = 320.0;
const TOOLTIP_VIEWPORT_MARGIN: f32 = 8.0;

fn tooltip_x(cursor_x: Pixels, tooltip_width: Pixels, viewport_width: Pixels) -> Pixels {
    let margin = px(TOOLTIP_VIEWPORT_MARGIN);
    let offset = px(12.0);
    let right_edge = (viewport_width - tooltip_width - margin).max(margin);
    let preferred = if cursor_x + offset + tooltip_width > viewport_width - margin {
        cursor_x - tooltip_width - offset
    } else {
        cursor_x + offset
    };

    preferred.max(margin).min(right_edge)
}

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
        let max_width =
            px(TOOLTIP_MAX_WIDTH).min(viewport.width - px(TOOLTIP_VIEWPORT_MARGIN * 2.0));

        // Estimated dimensions for boundary calculation
        let text_chars = self.text.chars().count() as f32;
        let est_width = px((text_chars * 7.5 + 20.0).max(40.0)).min(max_width);
        let est_height = px(26.0);

        let cursor_x = self.cursor_pos.x;
        let cursor_y = self.cursor_pos.y;

        let safe_x = tooltip_x(cursor_x, est_width, viewport.width);
        let text_content = if let Some((title, description)) = self.text.split_once('\n') {
            div()
                .debug_selector(|| "global_cursor_tooltip_text".into())
                .w_full()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(15.0))
                        .whitespace_normal()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .child(title.to_owned()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .line_height(px(15.0))
                        .whitespace_normal()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_muted)
                        .child(description.to_owned()),
                )
        } else {
            div()
                .debug_selector(|| "global_cursor_tooltip_text".into())
                .w_full()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .line_height(px(15.0))
                .whitespace_normal()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .child(self.text.clone())
        };

        // Y calculation: place above cursor (-30px), flip below (+18px) if hitting top of window / titlebar
        let pos_y = if cursor_y - est_height - px(6.0) < px(36.0) {
            cursor_y + px(18.0)
        } else {
            cursor_y - est_height - px(6.0)
        };

        // Safe clamping within window viewport
        let safe_y = pos_y
            .max(px(36.0))
            .min(viewport.height - est_height - px(8.0));

        div()
            .id(ElementId::Name("global_cursor_tooltip".into()))
            .debug_selector(|| "global_cursor_tooltip".into())
            .absolute()
            .top(safe_y)
            .left(safe_x)
            .flex()
            .items_center()
            .justify_center()
            .w(est_width)
            .overflow_hidden()
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
            .child(text_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, VisualTestContext, point, size};

    struct TooltipTest;

    impl Render for TooltipTest {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(Tooltip::new(
                "Побочный эффект · Низкий\nОчень длинное описание побочного эффекта, которое должно переноситься внутри тултипа, а не выходить за его границы",
                point(px(390.0), px(100.0)),
            ))
        }
    }

    #[test]
    fn tooltip_x_stays_inside_viewport() {
        assert_eq!(tooltip_x(px(4.0), px(320.0), px(900.0)), px(16.0));
        assert_eq!(tooltip_x(px(890.0), px(320.0), px(900.0)), px(558.0));
        assert_eq!(tooltip_x(px(100.0), px(320.0), px(300.0)), px(8.0));
    }

    #[gpui::test]
    fn tooltip_text_stays_inside_container(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(400.0), px(200.0)), |_, _| TooltipTest);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let tooltip = cx.debug_bounds("global_cursor_tooltip").unwrap();
        let text = cx.debug_bounds("global_cursor_tooltip_text").unwrap();

        assert!(text.left() >= tooltip.left());
        assert!(text.right() <= tooltip.right());
        assert!(tooltip.left() >= px(8.0));
        assert!(tooltip.right() <= px(392.0));
    }
}
