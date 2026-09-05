use gpui::{TestAppContext, px, size};

use crate::features::navigation::AppRoute;

use super::AppView;

#[gpui::test]
fn test_navigation_history(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(800.0), px(600.0)), |_window, _cx| AppView::new());

    window
        .update(cx, |view: &mut AppView, window, cx| {
            assert_eq!(view.current_route, AppRoute::Dashboard);
            assert!(view.history_back.is_empty());
            assert!(view.history_forward.is_empty());

            // Navigate to ContextMenu
            view.navigate_to(AppRoute::ContextMenu, window, cx);
            assert_eq!(view.current_route, AppRoute::ContextMenu);
            assert_eq!(view.history_back, vec![AppRoute::Dashboard]);
            assert!(view.history_forward.is_empty());

            // Navigate to Explorer
            view.navigate_to(AppRoute::Explorer, window, cx);
            assert_eq!(view.current_route, AppRoute::Explorer);
            assert_eq!(
                view.history_back,
                vec![AppRoute::Dashboard, AppRoute::ContextMenu]
            );

            // Back to ContextMenu
            view.navigate_back(window, cx);
            assert_eq!(view.current_route, AppRoute::ContextMenu);
            assert_eq!(view.history_back, vec![AppRoute::Dashboard]);
            assert_eq!(view.history_forward, vec![AppRoute::Explorer]);

            // Back to Dashboard
            view.navigate_back(window, cx);
            assert_eq!(view.current_route, AppRoute::Dashboard);
            assert!(view.history_back.is_empty());
            assert_eq!(
                view.history_forward,
                vec![AppRoute::Explorer, AppRoute::ContextMenu]
            );

            // Forward to ContextMenu
            view.navigate_forward(window, cx);
            assert_eq!(view.current_route, AppRoute::ContextMenu);
            assert_eq!(view.history_back, vec![AppRoute::Dashboard]);
            assert_eq!(view.history_forward, vec![AppRoute::Explorer]);

            // Forward to Explorer
            view.navigate_forward(window, cx);
            assert_eq!(view.current_route, AppRoute::Explorer);
            assert_eq!(
                view.history_back,
                vec![AppRoute::Dashboard, AppRoute::ContextMenu]
            );
            assert!(view.history_forward.is_empty());
        })
        .unwrap();
}

#[gpui::test]
fn test_escape_navigation_hierarchy(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(800.0), px(600.0)), |_window, _cx| AppView::new());

    window
        .update(cx, |view: &mut AppView, window, cx| {
            // Navigate to Cleanup (child of Tools)
            view.navigate_to(AppRoute::Cleanup, window, cx);
            assert_eq!(view.current_route, AppRoute::Cleanup);

            // Escape should go to Tools
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Tools);

            // Escape on Tools should go to Dashboard
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Dashboard);

            // Escape on Dashboard should stay on Dashboard
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Dashboard);

            // Navigate to CpuDetail (child of Dashboard)
            view.navigate_to(AppRoute::CpuDetail, window, cx);
            assert_eq!(view.current_route, AppRoute::CpuDetail);

            // Escape on CpuDetail should go to Dashboard
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Dashboard);
        })
        .unwrap();
}

#[gpui::test]
fn test_escape_prioritizes_dropdowns_and_search(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(800.0), px(600.0)), |_window, _cx| AppView::new());

    window
        .update(cx, |view: &mut AppView, window, cx| {
            // Navigate to Cleanup
            view.navigate_to(AppRoute::Cleanup, window, cx);

            // Simulate open dropdown
            view.open_dropdown = Some("test_dropdown");

            // Escape closes dropdown without navigating away from Cleanup
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Cleanup);

            // Simulate search with query
            view.startup_search_query = "search term".to_string();
            view.startup_search_focused = true;

            // First escape clears search query without navigating away
            view.handle_escape(window, cx);
            assert!(view.startup_search_query.is_empty());
            assert_eq!(view.current_route, AppRoute::Cleanup);

            // Second escape unfocuses search
            view.handle_escape(window, cx);
            assert!(!view.startup_search_focused);
            assert_eq!(view.current_route, AppRoute::Cleanup);

            // Third escape navigates up to Tools
            view.handle_escape(window, cx);
            assert_eq!(view.current_route, AppRoute::Tools);
        })
        .unwrap();
}