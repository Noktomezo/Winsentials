use std::time::Duration;

use gpui::Context;

use super::AppView;
use crate::entities::startup::{StartupEntry, StartupScope, StartupSource, StartupStatus};

impl AppView {
    pub fn refresh_startup_entries(&mut self, cx: &mut Context<Self>) {
        self.startup_entries = crate::entities::startup::fetch_all_startup_entries();
        cx.notify();
    }

    pub fn toggle_startup(&mut self, entry: &StartupEntry, cx: &mut Context<Self>) {
        crate::entities::startup::toggle_startup_entry(entry);
        self.refresh_startup_entries(cx);
    }

    pub fn delete_startup(&mut self, entry: &StartupEntry, cx: &mut Context<Self>) {
        crate::entities::startup::delete_startup_entry(entry);
        self.refresh_startup_entries(cx);
    }

    pub fn toggle_startup_filters(&mut self, cx: &mut Context<Self>) {
        if self.startup_filters_open {
            self.startup_filters_open = false;
            self.startup_filters_closing = true;
            self.startup_reset_closing = false;
            if let Some(open) = self.open_dropdown {
                if open.starts_with("startup_") {
                    self.start_closing_dropdown(open, cx);
                }
            }
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(140))
                    .await;
                this.update(cx, |this, cx| {
                    if this.startup_filters_closing {
                        this.startup_filters_closing = false;
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
        } else {
            self.startup_filters_open = true;
            self.startup_filters_closing = false;
            cx.notify();
        }
    }

    pub(crate) fn is_any_startup_filter_active(&self) -> bool {
        self.startup_scope_filter.is_some()
            || self.startup_filter.is_some()
            || self.startup_status_filter.is_some()
    }

    pub fn start_closing_startup_reset(&mut self, cx: &mut Context<Self>) {
        if self.startup_reset_closing {
            return;
        }
        self.startup_reset_closing = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(150))
                .await;
            this.update(cx, |this, cx| {
                if this.startup_reset_closing {
                    this.startup_reset_closing = false;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn set_startup_filter(&mut self, filter: Option<StartupSource>, cx: &mut Context<Self>) {
        let was_active = self.is_any_startup_filter_active();
        self.startup_filter = filter;
        self.startup_search_focused = false;
        self.start_closing_dropdown("startup_source", cx);
        if was_active && !self.is_any_startup_filter_active() {
            self.start_closing_startup_reset(cx);
        } else if self.is_any_startup_filter_active() {
            self.startup_reset_closing = false;
        }
        cx.notify();
    }

    pub fn set_startup_scope_filter(
        &mut self,
        scope: Option<StartupScope>,
        cx: &mut Context<Self>,
    ) {
        let was_active = self.is_any_startup_filter_active();
        self.startup_scope_filter = scope;
        self.startup_search_focused = false;
        self.start_closing_dropdown("startup_scope", cx);
        if was_active && !self.is_any_startup_filter_active() {
            self.start_closing_startup_reset(cx);
        } else if self.is_any_startup_filter_active() {
            self.startup_reset_closing = false;
        }
        cx.notify();
    }

    pub fn set_startup_status_filter(
        &mut self,
        status: Option<StartupStatus>,
        cx: &mut Context<Self>,
    ) {
        let was_active = self.is_any_startup_filter_active();
        self.startup_status_filter = status;
        self.startup_search_focused = false;
        self.start_closing_dropdown("startup_status", cx);
        if was_active && !self.is_any_startup_filter_active() {
            self.start_closing_startup_reset(cx);
        } else if self.is_any_startup_filter_active() {
            self.startup_reset_closing = false;
        }
        cx.notify();
    }

    pub fn reset_startup_filters(&mut self, cx: &mut Context<Self>) {
        let was_active = self.is_any_startup_filter_active();
        self.startup_scope_filter = None;
        self.startup_filter = None;
        self.startup_status_filter = None;
        self.close_dropdowns_if_startup(cx);
        if was_active {
            self.start_closing_startup_reset(cx);
        }
        cx.notify();
    }

    fn close_dropdowns_if_startup(&mut self, cx: &mut Context<Self>) {
        if let Some(open) = self.open_dropdown {
            if open.starts_with("startup_") {
                self.start_closing_dropdown(open, cx);
            }
        }
    }

    pub fn set_startup_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.startup_search_query = query;
        cx.notify();
    }

    pub fn set_startup_search_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        self.startup_search_hovered = hovered;
        cx.notify();
    }

    pub fn set_startup_search_focused(&mut self, focused: bool, cx: &mut Context<Self>) {
        self.startup_search_focused = focused;
        if !focused {
            self.startup_search_selection = None;
        }
        cx.notify();
    }

    pub fn set_startup_search_selection(
        &mut self,
        selection: Option<(usize, usize)>,
        cx: &mut Context<Self>,
    ) {
        self.startup_search_selection = selection;
        cx.notify();
    }

    pub fn set_startup_menu(&mut self, menu_id: Option<String>, cx: &mut Context<Self>) {
        self.startup_open_menu_id = menu_id;
        if self.startup_open_menu_id.is_some() {
            self.startup_search_focused = false;
            self.startup_search_selection = None;
        }
        cx.notify();
    }

    pub fn set_hovered_startup_card(&mut self, card_id: Option<String>, cx: &mut Context<Self>) {
        if self.hovered_startup_card != card_id {
            self.hovered_startup_card = card_id;
            cx.notify();
        }
    }
}
