use super::*;
use gpui::{
    Context, ElementId, IntoElement, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, px, size,
};

struct TestCheckboxView {
    checked: bool,
    indeterminate: bool,
    disabled: bool,
}

impl Render for TestCheckboxView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            Checkbox::new("test_checkbox")
                .checked(self.checked)
                .indeterminate(self.indeterminate)
                .disabled(self.disabled),
        )
    }
}

#[test]
fn test_checkbox_builder() {
    let cb = Checkbox::new("cb1")
        .checked(true)
        .indeterminate(false)
        .disabled(false);

    assert_eq!(cb.id, ElementId::Name("cb1".into()));
    assert!(cb.checked);
    assert!(!cb.indeterminate);
    assert!(!cb.disabled);

    let cb_ind = Checkbox::new("cb2")
        .checked(false)
        .indeterminate(true)
        .disabled(true);

    assert!(!cb_ind.checked);
    assert!(cb_ind.indeterminate);
    assert!(cb_ind.disabled);
}

#[gpui::test]
fn test_checkbox_render_unchecked(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(200.0), px(200.0)), |_window, _cx| {
        TestCheckboxView {
            checked: false,
            indeterminate: false,
            disabled: false,
        }
    });

    let mut visual_cx = VisualTestContext::from_window(window.into(), cx);
    visual_cx.run_until_parked();

    let bounds = visual_cx
        .debug_bounds("test_checkbox")
        .expect("checkbox must be present");
    assert_eq!(bounds.size.width, px(18.0));
    assert_eq!(bounds.size.height, px(18.0));
}

#[gpui::test]
fn test_checkbox_render_checked(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(200.0), px(200.0)), |_window, _cx| {
        TestCheckboxView {
            checked: true,
            indeterminate: false,
            disabled: false,
        }
    });

    let mut visual_cx = VisualTestContext::from_window(window.into(), cx);
    visual_cx.run_until_parked();

    let bounds = visual_cx
        .debug_bounds("test_checkbox")
        .expect("checkbox must be present");
    assert_eq!(bounds.size.width, px(18.0));
    assert_eq!(bounds.size.height, px(18.0));

    let overlay_bounds = visual_cx
        .debug_bounds("test_checkbox_overlay")
        .expect("overlay must be present when checked");
    assert_eq!(overlay_bounds.size.width, px(18.0));
    assert_eq!(overlay_bounds.size.height, px(18.0));
}

#[gpui::test]
fn test_checkbox_render_indeterminate(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(200.0), px(200.0)), |_window, _cx| {
        TestCheckboxView {
            checked: false,
            indeterminate: true,
            disabled: false,
        }
    });

    let mut visual_cx = VisualTestContext::from_window(window.into(), cx);
    visual_cx.run_until_parked();

    let bounds = visual_cx
        .debug_bounds("test_checkbox")
        .expect("checkbox must be present");
    assert_eq!(bounds.size.width, px(18.0));
    assert_eq!(bounds.size.height, px(18.0));

    let overlay_bounds = visual_cx
        .debug_bounds("test_checkbox_overlay")
        .expect("overlay must be present when indeterminate");
    assert_eq!(overlay_bounds.size.width, px(18.0));
    assert_eq!(overlay_bounds.size.height, px(18.0));
}

#[gpui::test]
fn test_checkbox_disabled(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(200.0), px(200.0)), |_window, _cx| {
        TestCheckboxView {
            checked: true,
            indeterminate: false,
            disabled: true,
        }
    });

    let mut visual_cx = VisualTestContext::from_window(window.into(), cx);
    visual_cx.run_until_parked();

    let bounds = visual_cx
        .debug_bounds("test_checkbox")
        .expect("checkbox must be present");
    assert_eq!(bounds.size.width, px(18.0));
}

#[gpui::test]
fn test_checkbox_toggle_interaction(cx: &mut TestAppContext) {
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestInteractiveCheckboxView {
        toggled: Arc<AtomicBool>,
    }

    impl Render for TestInteractiveCheckboxView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let toggled = self.toggled.clone();
            div().size_full().child(
                Checkbox::new("clickable_checkbox")
                    .checked(false)
                    .on_toggle(move |new_state, _w, _cx| {
                        toggled.store(new_state, Ordering::SeqCst);
                    }),
            )
        }
    }

    let toggled_state = Arc::new(AtomicBool::new(false));
    let toggled_clone = toggled_state.clone();

    let window = cx.open_window(size(px(200.0), px(200.0)), move |_window, _cx| {
        TestInteractiveCheckboxView {
            toggled: toggled_clone,
        }
    });

    let mut visual_cx = VisualTestContext::from_window(window.into(), cx);
    visual_cx.run_until_parked();

    let bounds = visual_cx
        .debug_bounds("clickable_checkbox")
        .expect("checkbox must be present");
    visual_cx.simulate_click(bounds.center(), gpui::Modifiers::default());
    visual_cx.run_until_parked();

    assert!(toggled_state.load(Ordering::SeqCst));
}
