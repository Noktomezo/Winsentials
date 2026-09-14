use std::time::Duration;

use gpui::{
    FontWeight, IntoElement, ParentElement, Pixels, Render, Rgba, SharedString, Styled,
    TestAppContext, VisualTestContext, Window, div, px, size,
};

use super::{MarqueeFadeMode, MarqueeText, marquee_ping_pong_easing, marquee_shift};

#[test]
fn short_text_does_not_shift() {
    assert_eq!(marquee_shift(px(60.0), px(100.0)), Pixels::ZERO);
    assert_eq!(marquee_shift(px(100.0), px(100.0)), Pixels::ZERO);
}

#[test]
fn overflowing_text_calculates_exact_travel() {
    assert_eq!(marquee_shift(px(160.0), px(100.0)), px(60.0));
}

#[test]
fn ping_pong_easing_symmetry() {
    assert_eq!(marquee_ping_pong_easing(0.0), 0.0);
    assert_eq!(marquee_ping_pong_easing(0.10), 0.0);
    assert_eq!(marquee_ping_pong_easing(0.50), 1.0);
    assert_eq!(marquee_ping_pong_easing(0.60), 1.0);
    assert_eq!(marquee_ping_pong_easing(1.0), 0.0);

    let mid_forward = marquee_ping_pong_easing(0.325);
    let mid_backward = marquee_ping_pong_easing(0.825);
    assert!((mid_forward - 0.5).abs() < 0.01);
    assert!((mid_backward - 0.5).abs() < 0.01);
}

#[test]
fn test_builder_and_fade_mode_compatibility() {
    let marquee = MarqueeText::new("test_id", "Sample Label", px(100.0))
        .font_size(px(14.0))
        .font_weight(FontWeight::BOLD)
        .text_color(Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
        .fade_color(Rgba {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        })
        .fade_width(px(10.0))
        .active(true)
        .fade_enabled(true)
        .fade_mode(MarqueeFadeMode::Opening(Duration::from_millis(160)))
        .duration(Duration::from_millis(2000))
        .debug_name("test_debug");

    assert_eq!(marquee.max_width, px(100.0));
    assert_eq!(marquee.font_size, px(14.0));
    assert_eq!(marquee.font_weight, FontWeight::BOLD);
    assert_eq!(marquee.fade_width, px(10.0));
    assert!(marquee.active);
    assert!(marquee.fade_enabled);
    assert_eq!(
        marquee.fade_mode,
        MarqueeFadeMode::Opening(Duration::from_millis(160))
    );
    assert_eq!(marquee.duration, Duration::from_millis(2000));
}

struct TestMarqueeView {
    text: SharedString,
    max_width: Pixels,
    active: bool,
}

impl Render for TestMarqueeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().size_full().p(px(20.0)).child(
            MarqueeText::new("test_marquee", self.text.clone(), self.max_width)
                .debug_name("test_marquee")
                .fade_width(px(8.0))
                .active(self.active),
        )
    }
}

#[gpui::test]
fn marquee_viewport_and_gutters_leave_text_unobscured_at_rest(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(400.0), px(200.0)), |_, _| TestMarqueeView {
        text: "Очень длинный текст пресета который заведомо переполняет контейнер".into(),
        max_width: px(120.0),
        active: false,
    });
    let mut cx = VisualTestContext::from_window(window.into(), cx);

    let anchor = cx.debug_bounds("test_marquee_anchor").unwrap();
    let viewport = cx.debug_bounds("test_marquee_viewport").unwrap();
    let line = cx.debug_bounds("test_marquee_line").unwrap();
    let fade_right = cx.debug_bounds("test_marquee_fade_right").unwrap();

    // 1. Anchor matches the constrained max width
    assert_eq!(anchor.size.width, px(120.0));

    // 2. Viewport expands into both gutters by fade_width (8px left, 8px right)
    assert_eq!(viewport.size.width, px(136.0));
    assert_eq!(viewport.left(), anchor.left() - px(8.0));
    assert_eq!(viewport.right(), anchor.right() + px(8.0));

    // 3. Text line at rest starts EXACTLY at anchor.left()
    assert_eq!(line.left(), anchor.left());

    // 4. Right fade gutter marker is placed strictly inside the right gutter [anchor.right(), anchor.right() + 8px]
    assert_eq!(fade_right.left(), anchor.right());
    assert_eq!(fade_right.size.width, px(8.0));
    assert_eq!(fade_right.right(), viewport.right());

    // 5. At rest, left fade marker is not attached so the first letter has zero fade
    assert!(cx.debug_bounds("test_marquee_fade_left").is_none());
}

#[gpui::test]
fn marquee_active_hover_places_left_fade_strictly_in_left_gutter(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(400.0), px(200.0)), |_, _| TestMarqueeView {
        text: "Очень длинный текст пресета который заведомо переполняет контейнер".into(),
        max_width: px(120.0),
        active: true,
    });
    let mut cx = VisualTestContext::from_window(window.into(), cx);

    let anchor = cx.debug_bounds("test_marquee_anchor").unwrap();
    let viewport = cx.debug_bounds("test_marquee_viewport").unwrap();
    let fade_left = cx.debug_bounds("test_marquee_fade_left").unwrap();

    // Left fade marker is placed strictly inside the left gutter [viewport.left(), anchor.left()]
    assert_eq!(fade_left.left(), viewport.left());
    assert_eq!(fade_left.right(), anchor.left());
    assert_eq!(fade_left.size.width, px(8.0));
}
