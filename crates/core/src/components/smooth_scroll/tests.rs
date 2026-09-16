use gpui::px;

use super::{SCROLL_DAMPING, damping_factor, offset_from_thumb, scroll_target, thumb_geometry};

#[test]
fn damping_and_thumb_geometry_stay_stable() {
    let sixty_fps = damping_factor(SCROLL_DAMPING, 1.0 / 60.0);
    assert!((sixty_fps - 0.095_162_57).abs() < 0.000_001);
    assert!(thumb_geometry(px(100.0), px(0.0), px(0.0)).is_none());

    let (top, height) = thumb_geometry(px(100.0), px(300.0), px(-150.0)).unwrap();
    assert_eq!(height, px(32.0));
    assert_eq!(top, px(30.0));
    assert_eq!(
        offset_from_thumb(px(45.0), px(15.0), px(92.0), px(32.0), px(300.0)),
        px(-150.0)
    );
    assert_eq!(scroll_target(px(0.0), px(20.0), px(300.0)), None);
    assert_eq!(scroll_target(px(-300.0), px(-20.0), px(300.0)), None);
    assert_eq!(
        scroll_target(px(0.0), px(-20.0), px(300.0)),
        Some(px(-20.0))
    );
}

#[test]
fn test_scroll_to_unmeasured_max_offset() {
    let mut state = super::SmoothScrollState::default();
    assert_eq!(state.handle.max_offset().y, px(0.0));

    // When max_offset is 0, scroll_to should still accept negative target_y
    let animating = state.scroll_to(px(-500.0), false);
    assert!(animating);
    assert_eq!(state.target_y, px(-500.0));

    // When reduce motion is enabled
    let mut rm_state = super::SmoothScrollState::default();
    let rm_animating = rm_state.scroll_to(px(-500.0), true);
    assert!(!rm_animating);
    assert_eq!(rm_state.target_y, px(-500.0));
}
