use gpui::{div, px, size, Context, IntoElement, ParentElement, Render, SharedString, Styled, TestAppContext, VisualTestContext, Window};
use super::*;

    struct TestDropdownView {
        open: bool,
        opening: bool,
        closing: bool,
        current_label: SharedString,
        width: Option<gpui::Pixels>,
    }

    impl Render for TestDropdownView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let mut dd = Dropdown::new("test_dd", self.current_label.clone(), "standard")
                .icon("icons/shield-check.svg")
                .options(vec![
                    ("standard", "╨б╤В╨░╨╜╨┤╨░╤А╤В", Some("icons/shield-check.svg")),
                    ("mild", "╨Ь╤П╨│╨║╨╕╨╣", Some("icons/feather.svg")),
                    (
                        "aggressive",
                        "╨Ю╤З╨╡╨╜╤М ╨┤╨╗╨╕╨╜╨╜╤Л╨╣ ╨░╨│╤А╨╡╤Б╤Б╨╕╨▓╨╜╤Л╨╣ ╨┐╤А╨╡╤Б╨╡╤В ╤Б ╨▒╨╛╨╗╤М╤И╨╕╨╝ ╨┐╨╡╤А╨╡╨┐╨╛╨╗╨╜╨╡╨╜╨╕╨╡╨╝",
                        Some("icons/flame.svg"),
                    ),
                ])
                .open(self.open)
                .opening(self.opening)
                .closing(self.closing);

            if let Some(w) = self.width {
                dd = dd.width(w);
            }

            div().size_full().p(px(20.0)).child(dd)
        }
    }

    #[gpui::test]
    fn dropdown_chevron_and_trigger_maintain_geometry_under_various_languages(
        cx: &mut TestAppContext,
    ) {
        // Test labels across Russian, English, and extra long labels
        let test_cases = [
            // Russian CTF presets
            ("╨б╤В╨░╨╜╨┤╨░╤А╤В", px(150.0)),
            ("╨Ь╤П╨│╨║╨╕╨╣", px(150.0)),
            ("╨Р╨│╤А╨╡╤Б╤Б╨╕╨▓╨╜╤Л╨╣", px(150.0)),
            // English CTF presets
            ("Standard", px(150.0)),
            ("Mild", px(150.0)),
            ("Aggressive", px(150.0)),
            // Russian Keyboard repeat presets
            ("╨б╨▓╨╡╤А╤Е╨▒╤Л╤Б╤В╤А╤Л╨╣", px(150.0)),
            // Long edge-case text to verify flex_none prevents chevron shrink
            (
                "╨Ю╤З╨╡╨╜╤М╨Ф╨╗╨╕╨╜╨╜╤Л╨╣╨в╨╡╨║╤Б╤В╨Ф╨╗╤П╨Я╤А╨╛╨▓╨╡╤А╨║╨╕╨Ю╨▓╨╡╤А╤Д╨╗╨╛╤Г╨С╨╡╨╖╨б╨╢╨░╤В╨╕╤П╨и╨╡╨▓╤А╨╛╨╜╨░",
                px(150.0),
            ),
            // Custom wider dropdown
            ("Wide Preset Mode", px(180.0)),
        ];

        for (label, expected_width) in test_cases {
            let window = cx.open_window(size(px(600.0), px(400.0)), move |_, _| TestDropdownView {
                open: false,
                opening: false,
                closing: false,
                current_label: label.into(),
                width: if expected_width != px(150.0) {
                    Some(expected_width)
                } else {
                    None
                },
            });
            let mut cx = VisualTestContext::from_window(window.into(), cx);

            let trigger_bounds = cx
                .debug_bounds("test_dd_trigger")
                .expect("trigger must be rendered");
            let chevron_bounds = cx
                .debug_bounds("test_dd_chevron_box")
                .expect("chevron must be rendered");

            // 1. Trigger maintains exact expected width
            assert_eq!(trigger_bounds.size.width, expected_width);

            // 2. Chevron is NEVER squished or shrunk below 14x14px regardless of text length
            assert_eq!(chevron_bounds.size.width, px(14.0));
            assert_eq!(chevron_bounds.size.height, px(14.0));

            // 3. Chevron stays strictly inside trigger bounds (no overflow)
            assert!(chevron_bounds.right() <= trigger_bounds.right());
            assert!(chevron_bounds.left() >= trigger_bounds.left());
        }
    }

    #[gpui::test]
    fn dropdown_menu_box_matches_width_and_options_do_not_overflow(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "╨б╤В╨░╨╜╨┤╨░╤А╤В".into(),
            width: None,
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_bounds = cx.debug_bounds("test_dd_trigger").unwrap();
        let menu_bounds = cx.debug_bounds("test_dd_menu_box").unwrap();

        // 1. Menu box width matches trigger width
        assert_eq!(menu_bounds.size.width, trigger_bounds.size.width);

        // 2. All options fit strictly inside menu box without horizontal overflow
        for (val, opt_selector) in [
            ("standard", "test_dd_opt_standard"),
            ("mild", "test_dd_opt_mild"),
            ("aggressive", "test_dd_opt_aggressive"),
        ] {
            let opt_bounds = cx
                .debug_bounds(opt_selector)
                .unwrap_or_else(|| panic!("option {val} must be rendered"));

            assert!(opt_bounds.left() >= menu_bounds.left());
            assert!(opt_bounds.right() <= menu_bounds.right());
        }

        // 3. Selected checkmark element is strictly 14px and doesn't squish
        let right_el_bounds = cx.debug_bounds("test_dd_opt_standard_right_el").unwrap();
        assert_eq!(right_el_bounds.size.width, px(14.0));
        assert_eq!(right_el_bounds.size.height, px(14.0));
    }

    #[gpui::test]
    fn dropdown_trigger_with_long_label_renders_marquee_with_fog(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: false,
            opening: false,
            closing: false,
            current_label: "╨Ю╤З╨╡╨╜╤М ╨┤╨╗╨╕╨╜╨╜╨╛╨╡ ╨╜╨░╨╖╨▓╨░╨╜╨╕╨╡ ╨░╤Г╨┤╨╕╨╛╤Г╤Б╤В╤А╨╛╨╣╤Б╤В╨▓╨░ ╨╕╨╗╨╕ ╨┐╤А╨╡╤Б╨╡╤В╨░ ╨┐╨╡╤А╨╡╨║╨╗╤О╤З╨╡╨╜╨╕╤П ╨║╨╗╨░╨▓╨╕╤И"
                .into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_bounds = cx.debug_bounds("test_dd_trigger").unwrap();
        let chevron_bounds = cx.debug_bounds("test_dd_chevron_box").unwrap();

        assert_eq!(trigger_bounds.size.width, px(150.0));
        assert_eq!(chevron_bounds.size.width, px(14.0));
        assert!(chevron_bounds.right() <= trigger_bounds.right());
    }

    #[gpui::test]
    fn dropdown_open_options_with_overflow_render_fog_correctly(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "╨б╤В╨░╨╜╨┤╨░╤А╤В".into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let menu_bounds = cx.debug_bounds("test_dd_menu_box").unwrap();
        let selected_opt = cx.debug_bounds("test_dd_opt_standard").unwrap();
        let checkmark = cx.debug_bounds("test_dd_opt_standard_right_el").unwrap();

        assert_eq!(menu_bounds.size.width, px(150.0));
        assert!(selected_opt.left() >= menu_bounds.left());
        assert!(selected_opt.right() <= menu_bounds.right());
        assert_eq!(checkmark.size.width, px(14.0));
        assert!(checkmark.right() <= selected_opt.right());
    }

    #[gpui::test]
    fn dropdown_trigger_fog_remains_present_when_menu_is_open(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "╨Ю╤З╨╡╨╜╤М ╨┤╨╗╨╕╨╜╨╜╨╛╨╡ ╨╜╨░╨╖╨▓╨░╨╜╨╕╨╡ ╨░╤Г╨┤╨╕╨╛╤Г╤Б╤В╤А╨╛╨╣╤Б╤В╨▓╨░ ╨╕╨╗╨╕ ╨┐╤А╨╡╤Б╨╡╤В╨░ ╨┐╨╡╤А╨╡╨║╨╗╤О╤З╨╡╨╜╨╕╤П ╨║╨╗╨░╨▓╨╕╤И"
                .into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_fog = cx
            .debug_bounds("test_dd_trigger_marquee_fade_right")
            .expect("trigger fog must remain visible even when menu is open");
        assert!(trigger_fog.size.width > px(0.0));
    }

    #[gpui::test]
    fn dropdown_animation_opening_and_closing_maintains_continuous_bounds(cx: &mut TestAppContext) {
        // 1. Opening phase: menu is marked open & opening
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: true,
            closing: false,
            current_label: "╨б╤В╨░╨╜╨┤╨░╤А╤В".into(),
            width: Some(px(160.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let menu_opening_bounds = cx
            .debug_bounds("test_dd_menu_box")
            .expect("menu box must exist during open animation");
        assert_eq!(menu_opening_bounds.size.width, px(160.0));

        let opt_marquee = cx
            .debug_bounds("test_dd_opt_marquee_aggressive_anchor")
            .expect("overflowing option marquee anchor must be rendered");
        assert!(opt_marquee.size.width > px(0.0));

        let opt_fog_opening = cx
            .debug_bounds("test_dd_opt_marquee_aggressive_fade_right")
            .expect("overflowing option fog must be rendered during open animation");
        assert!(opt_fog_opening.size.width > px(0.0));

        // 2. Closing phase: open is false, closing is true
        let window_close = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: false,
            opening: false,
            closing: true,
            current_label: "╨б╤В╨░╨╜╨┤╨░╤А╤В".into(),
            width: Some(px(160.0)),
        });
        let mut cx_close = VisualTestContext::from_window(window_close.into(), &cx);

        let menu_closing_bounds = cx_close
            .debug_bounds("test_dd_menu_box_close")
            .expect("menu box close must exist during close animation");
        assert_eq!(menu_closing_bounds.size.width, px(160.0));

        // Marquee anchor preserves identical width and coordinates during closing
        let opt_marquee_close = cx_close
            .debug_bounds("test_dd_opt_marquee_aggressive_anchor")
            .expect("overflowing option marquee anchor must remain stable during close animation");
        assert_eq!(opt_marquee_close.size.width, opt_marquee.size.width);

        let opt_fog_closing = cx_close
            .debug_bounds("test_dd_opt_marquee_aggressive_fade_right")
            .expect("overflowing option fog must be rendered during close animation");
        assert!(opt_fog_closing.size.width > px(0.0));
    }
