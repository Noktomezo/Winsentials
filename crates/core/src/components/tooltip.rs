use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Window, div, ease_in_out, px,
};

use crate::theme::Theme;

const TOOLTIP_MAX_WIDTH: f32 = 320.0;
const TOOLTIP_VIEWPORT_MARGIN: f32 = 8.0;
const TOOLTIP_TITLEBAR_HEIGHT: f32 = 36.0;

#[allow(clippy::cast_precision_loss)]
fn estimate_tooltip_size(text: &str, viewport_width: Pixels) -> (Pixels, Pixels) {
    let max_w_f32 =
        TOOLTIP_MAX_WIDTH.min(f32::from(viewport_width) - TOOLTIP_VIEWPORT_MARGIN * 2.0);
    let padding_x = 16.0; // 8px left + 8px right
    let content_max_w = (max_w_f32 - padding_x).max(20.0);

    if let Some((title, description)) = text.split_once('\n') {
        let mut title_lines = 0.0;
        let mut title_max_w: f32 = 0.0;
        for line in title.lines() {
            let line_chars = line.chars().count() as f32;
            let line_w = line_chars * 7.2;
            if line_w > title_max_w {
                title_max_w = line_w;
            }
            title_lines += (line_w / content_max_w).ceil().max(1.0);
        }
        if title_lines == 0.0 {
            title_lines = 1.0;
        }
        let title_h = title_lines * 15.0;

        let mut desc_lines = 0.0;
        let mut desc_max_w: f32 = 0.0;
        for line in description.lines() {
            let line_chars = line.chars().count() as f32;
            let line_w = line_chars * 6.0;
            if line_w > desc_max_w {
                desc_max_w = line_w;
            }
            desc_lines += (line_w / content_max_w).ceil().max(1.0);
        }
        if desc_lines == 0.0 {
            desc_lines = 1.0;
        }
        let desc_h = desc_lines * 15.0;

        let max_content_w = title_max_w.max(desc_max_w);
        let tooltip_width = px((max_content_w + padding_x).clamp(40.0, max_w_f32));
        // Title (title_h) + gap (2px) + desc (desc_h) + py(4px)*2 (8px) + border(1px)*2 (2px) = 12px chrome
        let tooltip_height = px(title_h + 2.0 + desc_h + 10.0);

        (tooltip_width, tooltip_height)
    } else {
        let chars = text.chars().count() as f32;
        let text_w = chars * 7.5;
        let tooltip_width = px((text_w + padding_x).clamp(36.0, max_w_f32));
        // Single line: py(4px)*2 (8px) + border(1px)*2 (2px) + line_height(15px) = 25px
        let tooltip_height = px(25.0);

        (tooltip_width, tooltip_height)
    }
}

fn tooltip_x(cursor_x: Pixels, tooltip_width: Pixels, viewport_width: Pixels) -> Pixels {
    let margin = px(TOOLTIP_VIEWPORT_MARGIN);
    let offset = px(12.0);
    let right_edge = (viewport_width - tooltip_width - margin).max(margin);
    let preferred = if cursor_x + offset + tooltip_width > viewport_width - margin {
        cursor_x - tooltip_width - offset
    } else {
        cursor_x + offset
    };

    preferred.clamp(margin, right_edge)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TooltipPlacementY {
    Above { bottom: Pixels },
    Below { top: Pixels },
}

fn tooltip_placement_y(
    cursor_y: Pixels,
    tooltip_height: Pixels,
    viewport_height: Pixels,
) -> TooltipPlacementY {
    let margin = px(TOOLTIP_VIEWPORT_MARGIN);
    let top_margin = px(TOOLTIP_TITLEBAR_HEIGHT);
    let offset = px(8.0);

    let effective_top_margin = if cursor_y < top_margin {
        margin
    } else {
        top_margin
    };

    let space_above = cursor_y - offset - effective_top_margin;
    let space_below = viewport_height - margin - (cursor_y + px(20.0));

    let place_above = if space_above >= tooltip_height {
        true
    } else if space_below >= tooltip_height {
        false
    } else {
        space_above >= space_below
    };

    if place_above {
        let preferred_bottom = viewport_height - (cursor_y - offset);
        let max_bottom = (viewport_height - effective_top_margin - tooltip_height).max(px(0.0));
        TooltipPlacementY::Above {
            bottom: preferred_bottom.min(max_bottom),
        }
    } else {
        let preferred_top = cursor_y + px(20.0);
        let max_top = (viewport_height - margin - tooltip_height).max(effective_top_margin);
        TooltipPlacementY::Below {
            top: preferred_top.min(max_top),
        }
    }
}

#[cfg(test)]
fn tooltip_y(cursor_y: Pixels, tooltip_height: Pixels, viewport_height: Pixels) -> Pixels {
    match tooltip_placement_y(cursor_y, tooltip_height, viewport_height) {
        TooltipPlacementY::Above { bottom } => viewport_height - bottom - tooltip_height,
        TooltipPlacementY::Below { top } => top,
    }
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

        let (est_width, est_height) = estimate_tooltip_size(&self.text, viewport.width);

        let cursor_x = self.cursor_pos.x;
        let cursor_y = self.cursor_pos.y;

        let safe_x = tooltip_x(cursor_x, est_width, viewport.width);
        let placement = tooltip_placement_y(cursor_y, est_height, viewport.height);
        let is_above = matches!(placement, TooltipPlacementY::Above { .. });
        let (is_multiline, text_content) =
            if let Some((title, description)) = self.text.split_once('\n') {
                (
                    true,
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
                        ),
                )
            } else {
                (
                    false,
                    div()
                        .debug_selector(|| "global_cursor_tooltip_text".into())
                        .whitespace_nowrap()
                        .text_size(px(12.0))
                        .line_height(px(15.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_primary)
                        .child(self.text.clone()),
                )
            };

        let mut container = div()
            .id(ElementId::Name("global_cursor_tooltip".into()))
            .debug_selector(|| "global_cursor_tooltip".into())
            .absolute()
            .left(safe_x)
            .max_w(max_width)
            .max_h((viewport.height - px(TOOLTIP_VIEWPORT_MARGIN * 2.0)).max(px(28.0)))
            .overflow_hidden()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.input_border)
            .bg(theme.input_bg)
            .shadow_md();

        match placement {
            TooltipPlacementY::Above { bottom } => {
                container = container.bottom(bottom);
            }
            TooltipPlacementY::Below { top } => {
                container = container.top(top);
            }
        }

        if is_multiline {
            container = container.w(est_width).flex().flex_col().items_start();
        } else {
            container = container.flex().items_center().whitespace_nowrap();
        }

        container
            .with_animation(
                ElementId::Name("tooltip_enter".into()),
                Animation::new(Duration::from_millis(120)).with_easing(ease_in_out),
                move |box_el, delta| {
                    let opacity = delta;
                    let offset_y = (1.0 - delta) * 3.0;
                    if is_above {
                        box_el.opacity(opacity).mb(px(offset_y))
                    } else {
                        box_el.opacity(opacity).mt(px(offset_y))
                    }
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

    struct BottomEdgeTooltipTest;

    impl Render for BottomEdgeTooltipTest {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(Tooltip::new(
                "Побочный эффект · Внимание\nSnapKey заменяет аппаратный ввод через низкоуровневый хук. В играх с автодетектом Snap Tap (например Counter-Strike 2) вы можете получить исключение из матча ('Kicked for input automation'). В некоторых соревновательных ритм-играх (например osu!) любые скрипты автоматизации строго запрещены и могут привести к перманентной блокировке аккаунта.",
                point(px(400.0), px(580.0)),
            ))
        }
    }

    #[test]
    fn tooltip_x_stays_inside_viewport() {
        assert_eq!(tooltip_x(px(4.0), px(320.0), px(900.0)), px(16.0));
        assert_eq!(tooltip_x(px(890.0), px(320.0), px(900.0)), px(558.0));
        assert_eq!(tooltip_x(px(100.0), px(320.0), px(300.0)), px(8.0));
    }

    #[test]
    fn tooltip_y_stays_inside_viewport() {
        // Cursor near top: must place below and stay >= top_margin
        let y_top = tooltip_y(px(20.0), px(60.0), px(600.0));
        assert!(y_top >= px(8.0));

        // Cursor near bottom: must flip above cursor
        let y_bottom = tooltip_y(px(580.0), px(140.0), px(600.0));
        assert!(y_bottom <= px(580.0) - px(140.0));
        assert!(y_bottom >= px(36.0));

        // Cursor in middle with plenty of space: stays within bounds
        let y_mid = tooltip_y(px(300.0), px(50.0), px(600.0));
        assert!(y_mid >= px(36.0));
        assert!(y_mid + px(50.0) <= px(592.0));
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

    #[gpui::test]
    fn tooltip_stays_inside_viewport_at_bottom_edge(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(800.0), px(600.0)), |_, _| BottomEdgeTooltipTest);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let tooltip = cx.debug_bounds("global_cursor_tooltip").unwrap();

        assert!(
            tooltip.top() >= px(36.0),
            "Tooltip top {:?} should not be under titlebar",
            tooltip.top()
        );
        assert!(
            tooltip.bottom() <= px(600.0) - px(8.0),
            "Tooltip bottom {:?} should stay inside viewport bounds",
            tooltip.bottom()
        );
    }

    struct SingleLineTooltipTest;

    impl Render for SingleLineTooltipTest {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .relative()
                .size_full()
                .child(Tooltip::new("Ввод", point(px(24.0), px(200.0))))
        }
    }

    #[gpui::test]
    fn single_line_tooltip_does_not_wrap(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(400.0), px(300.0)), |_, _| SingleLineTooltipTest);
        let mut cx = VisualTestContext::from_window(window.into(), cx);
        let tooltip = cx.debug_bounds("global_cursor_tooltip").unwrap();
        let text = cx.debug_bounds("global_cursor_tooltip_text").unwrap();

        assert!(text.left() >= tooltip.left());
        assert!(text.right() <= tooltip.right());
        // Single line height: py(4px) * 2 + 15px line-height + 2px border = 25px
        assert!(
            tooltip.size.height <= px(28.0),
            "Single-line tooltip should not wrap into multiple lines, got height {:?}",
            tooltip.size.height
        );
    }

    struct ShortMultilineTooltipTest;

    impl Render for ShortMultilineTooltipTest {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(Tooltip::new(
                "Заголовок\nОдна короткая строка описания.",
                point(px(300.0), px(400.0)),
            ))
        }
    }

    struct LongMultilineTooltipTest;

    impl Render for LongMultilineTooltipTest {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(Tooltip::new(
                "Заголовок\nПервая строка очень длинного подробного описания компонента.\nВторая строка подробного описания.\nТретья дополнительная строка текста.",
                point(px(300.0), px(400.0)),
            ))
        }
    }

    #[gpui::test]
    fn tooltip_bottom_anchored_regardless_of_line_count(cx: &mut TestAppContext) {
        let window_short =
            cx.open_window(size(px(800.0), px(600.0)), |_, _| ShortMultilineTooltipTest);
        let mut cx_short = VisualTestContext::from_window(window_short.into(), cx);
        let short_bounds = cx_short.debug_bounds("global_cursor_tooltip").unwrap();

        let window_long =
            cx.open_window(size(px(800.0), px(600.0)), |_, _| LongMultilineTooltipTest);
        let mut cx_long = VisualTestContext::from_window(window_long.into(), cx);
        let long_bounds = cx_long.debug_bounds("global_cursor_tooltip").unwrap();

        assert_eq!(
            short_bounds.bottom(),
            long_bounds.bottom(),
            "Bottom of tooltip must remain anchored regardless of line count (short: {:?}, long: {:?})",
            short_bounds.bottom(),
            long_bounds.bottom()
        );
        // At initial animation frame (delta = 0), offset_y is 3.0px, so bottom is 400 - 8 - 3 = 389px
        assert_eq!(short_bounds.bottom(), px(389.0));
        assert!(long_bounds.size.height > short_bounds.size.height);
    }
}
