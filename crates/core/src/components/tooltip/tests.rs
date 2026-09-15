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
fn tooltip_placement_x_anchors_correctly() {
    assert_eq!(
        tooltip_placement_x(px(100.0), px(150.0), px(1000.0)),
        TooltipPlacementX::Right { left: px(112.0) }
    );
    assert_eq!(
        tooltip_placement_x(px(950.0), px(150.0), px(1000.0)),
        TooltipPlacementX::Left { right: px(62.0) }
    );
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
    let window = cx.open_window(size(px(800.0), px(600.0)), |_, _| ShortMultilineTooltipTest);
    let mut cx_short = VisualTestContext::from_window(window.into(), cx);
    let short_bounds = cx_short.debug_bounds("global_cursor_tooltip").unwrap();

    let window_long = cx.open_window(size(px(800.0), px(600.0)), |_, _| LongMultilineTooltipTest);
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
