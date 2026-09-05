use super::*;
use gpui::{point, px, rgba};

    #[test]
    fn perf_monitor_initial_metrics() {
        let mut state = DevPerfMonitorState::new();
        assert!(state.enabled);
        assert!(state.minimized);
        assert_eq!(state.hovered_control, None);
        state.set_hovered_control(Some("expand"));
        assert_eq!(state.hovered_control, Some("expand"));
        assert_eq!(state.snapshot().hovered_control, Some("expand"));
        state.record_frame(2.0);
        assert!(state.displayed_fps > 0.0);
    }

    #[test]
    fn perf_monitor_vsync_tolerance() {
        // 58-60 fps on a 60Hz display is healthy
        let healthy = fps_status_color(58.5);
        assert_eq!(healthy, rgba(0x10B9_81FF));

        // 45 fps is warning
        let warning = fps_status_color(45.0);
        assert_eq!(warning, rgba(0xF59E_0BFF));

        // 20 fps is critical
        let critical = fps_status_color(20.0);
        assert_eq!(critical, rgba(0xEF44_44FF));

        // 0 fps is idle
        let idle = fps_status_color(0.0);
        assert_eq!(idle, rgba(0x8888_88FF));
    }

    #[test]
    fn perf_monitor_bounds_clamping() {
        let mut state = DevPerfMonitorState::new();
        state.position = Some(point(px(-100.0), px(-50.0)));
        let clamped = state.current_pos(px(800.0), px(600.0));
        assert!(clamped.x >= px(BOUNDS_PADDING));
        assert!(clamped.y >= px(TITLEBAR_HEIGHT));
    }
