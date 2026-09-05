use gpui::{Context, Window};

use crate::features::navigation::AppRoute;
use crate::shared::ui::TooltipState;

use super::AppView;

impl AppView {
    pub fn toggle_sidebar(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_expanded = !self.sidebar_expanded;
        self.active_tooltip = None;
        cx.notify();
    }

    pub fn navigate_to(&mut self, route: AppRoute, _window: &mut Window, cx: &mut Context<Self>) {
        if self.current_route != route {
            self.history_back.push(self.current_route);
            if self.history_back.len() > 50 {
                self.history_back.remove(0);
            }
            self.history_forward.clear();
            self.set_route_internal(route, cx);
        }
    }

    pub fn navigate_back(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prev) = self.history_back.pop() {
            self.history_forward.push(self.current_route);
            if self.history_forward.len() > 50 {
                self.history_forward.remove(0);
            }
            self.set_route_internal(prev, cx);
        }
    }

    pub fn navigate_forward(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next) = self.history_forward.pop() {
            self.history_back.push(self.current_route);
            if self.history_back.len() > 50 {
                self.history_back.remove(0);
            }
            self.set_route_internal(next, cx);
        }
    }

    pub fn navigate_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(parent) = self.current_route.parent() {
            self.navigate_to(parent, window, cx);
        }
    }

    pub fn handle_escape(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open_dropdown.is_some() {
            self.close_dropdowns(window, cx);
            return;
        }
        if self.startup_open_menu_id.is_some() {
            self.startup_open_menu_id = None;
            cx.notify();
            return;
        }
        if !self.startup_search_query.is_empty() {
            self.startup_search_query.clear();
            cx.notify();
            return;
        }
        if self.startup_search_focused {
            self.startup_search_focused = false;
            self.startup_search_selection = None;
            if let Some(ref f) = self.focus_handle {
                f.focus(window, cx);
            }
            cx.notify();
            return;
        }
        self.navigate_up(window, cx);
    }

    pub(crate) fn set_route_internal(&mut self, route: AppRoute, cx: &mut Context<Self>) {
        self.current_route = route;
        if let Ok(mut mgr) = self.discord_rpc_manager.lock() {
            mgr.set_route(route);
        }
        self.open_dropdown = None;
        self.closing_dropdown = None;
        self.hovered_dropdown = None;
        self.hovered_option = None;
        self.pending_selection = None;
        self.hovered_telemetry_card = None;
        self.hovered_titlebar_breadcrumb = None;
        self.active_tooltip = None;
        self.startup_search_focused = false;
        self.startup_search_selection = None;
        self.startup_open_menu_id = None;
        if route == AppRoute::Startup {
            self.startup_entries = crate::entities::startup::fetch_all_startup_entries();
        }
        if route == AppRoute::Cleanup && !self.cleanup.scanned_once {
            self.refresh_cleanup(cx);
        }
        cx.notify();
    }

    pub fn set_hovered_route(
        &mut self,
        route: AppRoute,
        is_hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_route != Some(route) {
                self.hovered_route = Some(route);
                cx.notify();
            }
        } else if self.hovered_route == Some(route) {
            self.hovered_route = None;
            cx.notify();
        }
    }

    pub fn set_hovered_titlebar_breadcrumb(
        &mut self,
        id: &'static str,
        is_hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_titlebar_breadcrumb != Some(id) {
                self.hovered_titlebar_breadcrumb = Some(id);
                cx.notify();
            }
        } else if self.hovered_titlebar_breadcrumb == Some(id) {
            self.hovered_titlebar_breadcrumb = None;
            cx.notify();
        }
    }

    pub fn set_active_tooltip(&mut self, tooltip: Option<TooltipState>, cx: &mut Context<Self>) {
        if self.active_tooltip != tooltip {
            self.active_tooltip = tooltip;
            cx.notify();
        }
    }
}