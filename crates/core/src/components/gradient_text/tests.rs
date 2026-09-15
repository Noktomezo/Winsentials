use super::*;
use gpui::{Context, Render, TestAppContext, VisualTestContext, px, rgb, size};

struct TestGradientTextView;

impl Render for TestGradientTextView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        GradientText::new("test_gradient_title", "WINSENTIALS")
            .debug_selector("test_gradient_title")
            .font_family("Permanent Marker")
            .font_size(px(28.0))
            .letter_spacing(px(8.4))
            .colors(vec![TAILWIND_BLUE_400, TAILWIND_BLUE_600])
            .animated(false)
    }
}

#[test]
fn test_lerp_color() {
    let c1 = rgb(0x0000_0000);
    let c2 = rgb(0x00ff_ffff);

    let start = lerp_color(c1, c2, 0.0);
    assert!((start.r - 0.0).abs() < f32::EPSILON);
    assert!((start.g - 0.0).abs() < f32::EPSILON);
    assert!((start.b - 0.0).abs() < f32::EPSILON);

    let end = lerp_color(c1, c2, 1.0);
    assert!((end.r - 1.0).abs() < f32::EPSILON);
    assert!((end.g - 1.0).abs() < f32::EPSILON);
    assert!((end.b - 1.0).abs() < f32::EPSILON);

    let mid = lerp_color(c1, c2, 0.5);
    assert!((mid.r - 0.5).abs() < f32::EPSILON);
    assert!((mid.g - 0.5).abs() < f32::EPSILON);
    assert!((mid.b - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_multi_stop_color() {
    let empty: [Rgba; 0] = [];
    assert_eq!(multi_stop_color(&empty, 0.5), Rgba::default());

    let single = [TAILWIND_BLUE_400];
    assert_eq!(multi_stop_color(&single, 0.5), TAILWIND_BLUE_400);

    let two = [TAILWIND_BLUE_400, TAILWIND_BLUE_600];
    assert_eq!(multi_stop_color(&two, 0.0), TAILWIND_BLUE_400);
    assert_eq!(multi_stop_color(&two, 1.0), TAILWIND_BLUE_600);

    let three = [TAILWIND_BLUE_400, rgb(0x00ff_ffff), TAILWIND_BLUE_600];
    let mid = multi_stop_color(&three, 0.5);
    assert!((mid.r - 1.0).abs() < 1e-4);
}

#[test]
fn test_yoyo_progress() {
    let p0 = yoyo_progress(0.0);
    assert!((p0 - 0.0).abs() < 1e-5);

    let p_half = yoyo_progress(0.5);
    assert!((p_half - 1.0).abs() < 1e-5);

    let p1 = yoyo_progress(1.0);
    assert!((p1 - 0.0).abs() < 1e-5);

    let p_quarter = yoyo_progress(0.25);
    let p_three_quarter = yoyo_progress(0.75);
    assert!((p_quarter - p_three_quarter).abs() < 1e-5);
    assert!(p_quarter > 0.0 && p_quarter < 1.0);
}

#[test]
fn test_character_sample_pos() {
    let single = character_sample_pos(0, 1, 0.5, 0.35);
    assert!((single - 0.5).abs() < 1e-4);

    let first = character_sample_pos(0, 11, 0.0, 0.35);
    let last = character_sample_pos(10, 11, 0.0, 0.35);
    assert!(first < last);

    let first_shifted = character_sample_pos(0, 11, 1.0, 0.35);
    assert!(first_shifted > first);
}

#[test]
fn test_seamless_two_stop_color() {
    let c1 = TAILWIND_BLUE_400;
    let c2 = TAILWIND_BLUE_600;

    let start = seamless_two_stop_color(c1, c2, 0.0);
    let end = seamless_two_stop_color(c1, c2, 1.0);
    assert!((start.r - end.r).abs() < 1e-5);
    assert!((start.g - end.g).abs() < 1e-5);
    assert!((start.b - end.b).abs() < 1e-5);

    let mid = seamless_two_stop_color(c1, c2, 0.5);
    assert!((mid.r - c2.r).abs() < 1e-5);
    assert!((mid.g - c2.g).abs() < 1e-5);
    assert!((mid.b - c2.b).abs() < 1e-5);
}

#[test]
fn test_character_seamless_phase() {
    let phase_0 = character_seamless_phase(0, 11, 0.0);
    let phase_last = character_seamless_phase(10, 11, 0.0);
    assert!((phase_0 - 0.0).abs() < 1e-5);
    assert!((phase_last - 0.5).abs() < 1e-5);

    // Full loop continuity: phase at progress 1.0 matches progress 0.0
    let phase_wrap = character_seamless_phase(0, 11, 1.0);
    assert!((phase_wrap - phase_0).abs() < 1e-5);
}

#[test]
fn test_gradient_text_builder() {
    let default_gt = GradientText::new("def", "TEXT");
    assert!(!default_gt.yoyo);

    let gt = GradientText::new("custom_title", "TEST")
        .font_family("CustomFont")
        .font_size(px(32.0))
        .font_weight(FontWeight::BOLD)
        .letter_spacing(px(10.0))
        .colors(vec![rgb(0xff0000), rgb(0x0000ff)])
        .duration(Duration::from_secs(5))
        .shift_amplitude(0.4)
        .direction(GradientDirection::Horizontal)
        .yoyo(true)
        .animated(false);

    assert_eq!(gt.font_family.as_deref(), Some("CustomFont"));
    assert_eq!(gt.font_size, px(32.0));
    assert_eq!(gt.font_weight, FontWeight::BOLD);
    assert_eq!(gt.letter_spacing, Some(px(10.0)));
    assert_eq!(gt.duration, Duration::from_secs(5));
    assert!((gt.shift_amplitude - 0.4).abs() < 1e-5);
    assert_eq!(gt.direction, GradientDirection::Horizontal);
    assert!(gt.yoyo);
    assert!(!gt.animated);
}

#[gpui::test]
fn test_gradient_text_visual_render(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(500.0), px(100.0)), |_window, _cx| {
        TestGradientTextView
    });

    let mut cx = VisualTestContext::from_window(window.into(), cx);
    cx.run_until_parked();

    let bounds = cx
        .debug_bounds("test_gradient_title")
        .expect("GradientText must be rendered in window hierarchy");

    assert!(bounds.size.width > px(0.0));
    assert!(bounds.size.height > px(0.0));
}
