    use gpui::px;

    use super::{
        graph_percent_color, smooth_percent_transition, stepped_corner_radius,
        stepped_history_index,
    };
    use crate::theme::Theme;

    #[test]
    fn graph_colors_blend_between_dashboard_zones() {
        let theme = Theme::dark();

        assert_eq!(graph_percent_color(20.0, &theme), theme.accent_green);
        assert_eq!(graph_percent_color(70.0, &theme), theme.accent_yellow);
        assert_eq!(graph_percent_color(95.0, &theme), theme.accent_red);
    }

    #[test]
    fn maps_cursor_to_stepped_history_segment() {
        assert_eq!(stepped_history_index(0.2, 5), 0);
        assert_eq!(stepped_history_index(2.0, 5), 1);
        assert_eq!(stepped_history_index(4.8, 5), 4);
        assert_eq!(stepped_history_index(9.0, 5), 4);
    }

    #[test]
    fn core_transition_is_smooth_and_clamped() {
        assert_eq!(smooth_percent_transition(10.0, 50.0, -1.0), 10.0);
        assert_eq!(smooth_percent_transition(10.0, 50.0, 0.5), 30.0);
        assert_eq!(smooth_percent_transition(10.0, 50.0, 2.0), 50.0);
    }

    #[test]
    fn rounded_step_radius_stays_inside_short_transitions() {
        let radius = stepped_corner_radius(px(3.0), px(10.0), px(12.0));
        assert!(radius.width <= px(1.5));
        assert!(radius.height <= px(1.0));
    }
