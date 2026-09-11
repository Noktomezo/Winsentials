use std::sync::Arc;

use gpui::{
    App, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::entities::startup::search::matches_startup_query;
use crate::entities::startup::{StartupEntry, StartupStatus};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;
use crate::shared::ui::search_input::{SearchChangeHandler, SearchInput};
use crate::shared::ui::smooth_scroll::SmoothVirtualList;

pub mod card;
pub mod filter;
pub mod types;

pub(crate) use card::*;
pub(crate) use filter::*;
pub use types::*;
#[derive(IntoElement)]
pub struct StartupPage {
    entries: Vec<StartupEntry>,
    filter_state: StartupFilterState,
    dropdown_state: StartupDropdownState,
    search_query: String,
    search_focused: bool,
    search_hovered: bool,
    search_selection: Option<(usize, usize)>,
    open_menu_id: Option<String>,
    hovered_card_id: Option<String>,
    search_focus: Option<FocusHandle>,
    on_toggle: Option<StartupToggleHandler>,
    on_delete: Option<StartupDeleteHandler>,
    on_open_folder: Option<StartupActionHandler>,
    on_open_source: Option<StartupActionHandler>,
    on_copy_path: Option<StartupActionHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
    on_toggle_menu: Option<MenuToggleHandler>,
    filter_handlers: StartupFilterHandlers,
    on_change_search: Option<SearchChangeHandler>,
    on_hover_search: Option<SearchHoverHandler>,
    on_focus_search: Option<SearchFocusHandler>,
    on_selection_search: Option<SearchSelectionHandler>,
    on_hover_card: Option<StartupHoverCardHandler>,
}

impl StartupPage {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entries: Vec<StartupEntry>,
        filter_state: StartupFilterState,
        search_query: impl Into<String>,
        search_focused: bool,
        search_hovered: bool,
        search_selection: Option<(usize, usize)>,
        open_menu_id: Option<String>,
        hovered_card_id: Option<String>,
    ) -> Self {
        Self {
            entries,
            filter_state,
            dropdown_state: StartupDropdownState::default(),
            search_query: search_query.into(),
            search_focused,
            search_hovered,
            search_selection,
            open_menu_id,
            hovered_card_id,
            search_focus: None,
            on_toggle: None,
            on_delete: None,
            on_open_folder: None,
            on_open_source: None,
            on_copy_path: None,
            on_hover_tooltip: None,
            on_toggle_menu: None,
            filter_handlers: StartupFilterHandlers::default(),
            on_change_search: None,
            on_hover_search: None,
            on_focus_search: None,
            on_selection_search: None,
            on_hover_card: None,
        }
    }

    #[must_use]
    pub fn dropdown_state(mut self, state: StartupDropdownState) -> Self {
        self.dropdown_state = state;
        self
    }

    #[must_use]
    pub fn filter_handlers(mut self, handlers: StartupFilterHandlers) -> Self {
        self.filter_handlers = handlers;
        self
    }

    #[must_use]
    pub fn on_hover_card(
        mut self,
        handler: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_card = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn search_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.search_focus = Some(focus_handle.clone());
        self
    }

    #[must_use]
    pub fn on_change_search(
        mut self,
        handler: impl Fn(String, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_search = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_search(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_search = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_focus_search(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_focus_search = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_selection_search(
        mut self,
        handler: impl Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_search = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle(
        mut self,
        handler: impl Fn(&StartupEntry, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_delete(
        mut self,
        handler: impl Fn(&StartupEntry, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_delete = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_open_folder(
        mut self,
        handler: impl Fn(&StartupEntry, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_folder = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_open_source(
        mut self,
        handler: impl Fn(&StartupEntry, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_source = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_copy_path(
        mut self,
        handler: impl Fn(&StartupEntry, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_copy_path = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_tooltip(
        mut self,
        handler: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_menu(
        mut self,
        handler: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_menu = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for StartupPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Startup;

        let total_count = self.entries.len();
        let enabled_count = self
            .entries
            .iter()
            .filter(|e| e.status == StartupStatus::Enabled)
            .count();
        let disabled_count = total_count.saturating_sub(enabled_count);

        let filtered_entries: Vec<StartupEntry> = self
            .entries
            .into_iter()
            .filter(|e| {
                if let Some(scope) = self.filter_state.scope {
                    if e.scope != scope {
                        return false;
                    }
                }
                if let Some(source) = self.filter_state.source {
                    if e.source != source {
                        return false;
                    }
                }
                if let Some(status) = self.filter_state.status {
                    if e.status != status {
                        return false;
                    }
                }
                matches_startup_query(e, &self.search_query)
            })
            .collect();

        let handlers = StartupCardHandlers {
            toggle: self.on_toggle,
            delete: self.on_delete,
            open_folder: self.on_open_folder,
            open_source: self.on_open_source,
            copy_path: self.on_copy_path,
            hover_tt: self.on_hover_tooltip.clone(),
            toggle_menu: self.on_toggle_menu,
            hover_card: self.on_hover_card,
            hovered_card_id: self.hovered_card_id,
        };

        let tt_handler = self.on_hover_tooltip.clone();
        let tooltip_msg = rust_i18n::t!(
            "startup.badge_tooltip",
            total = total_count,
            enabled = enabled_count,
            disabled = disabled_count
        )
        .to_string();

        let count_badge = div()
            .id(ElementId::Name("startup_count_badge".into()))
            .flex()
            .items_center()
            .justify_center()
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(6.0))
            .bg(theme.button_selected)
            .border_1()
            .border_color(theme.card_border)
            .text_size(px(12.0))
            .font_weight(FontWeight::BOLD)
            .text_color(theme.text_primary)
            .cursor_pointer()
            .hover(move |s| s.bg(theme.button_hover))
            .on_mouse_move({
                let tt_h = tt_handler.clone();
                let tt_text = tooltip_msg.clone();
                move |event, window, cx| {
                    if let Some(ref h) = tt_h {
                        h(
                            Some(TooltipState {
                                text: tt_text.clone().into(),
                                cursor_pos: event.position,
                            }),
                            window,
                            cx,
                        );
                    }
                }
            })
            .on_hover({
                let tt_h = tt_handler.clone();
                move |hovered, window, cx| {
                    if !hovered {
                        if let Some(ref h) = tt_h {
                            h(None, window, cx);
                        }
                    }
                }
            })
            .child(format!("{total_count}"));

        let mut search_input = SearchInput::new("startup_search", &self.search_query)
            .w_full()
            .focused(self.search_focused)
            .hovered(self.search_hovered)
            .selection(self.search_selection);
        if let Some(ref f) = self.search_focus {
            search_input = search_input.track_focus(f);
        }
        if let Some(ref h) = self.on_change_search {
            let h_clone = h.clone();
            search_input = search_input.on_change(move |q, window, cx| {
                h_clone(q, window, cx);
            });
        }
        if let Some(ref h) = self.on_hover_search {
            let h_clone = h.clone();
            search_input = search_input.on_hover(move |hov, window, cx| {
                h_clone(hov, window, cx);
            });
        }
        if let Some(ref h) = self.on_focus_search {
            let h_clone = h.clone();
            search_input = search_input.on_focus_change(move |foc, window, cx| {
                h_clone(foc, window, cx);
            });
        }
        if let Some(ref h) = self.on_selection_search {
            let h_clone = h.clone();
            search_input = search_input.on_selection_change(move |sel, window, cx| {
                h_clone(sel, window, cx);
            });
        }

        let filter_bar = render_filter_bar(
            search_input,
            self.filter_state,
            self.dropdown_state,
            &self.filter_handlers,
        );

        let total_items = filtered_entries.len();
        let entries_arc = Arc::new(filtered_entries);
        let entries_render = entries_arc.clone();
        let open_menu = self.open_menu_id;
        let card_theme = theme;

        SmoothVirtualList::new(
            route.id(),
            total_items,
            px(58.0),
            px(8.0),
            move |idx, _window, _cx| {
                if let Some(entry) = entries_render.get(idx) {
                    let is_menu_open = open_menu.as_ref().is_some_and(|id| id == &entry.id);
                    render_startup_card(entry, &card_theme, is_menu_open, &handlers)
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            },
        )
        .header(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .w_full()
                .child(PageHeader::new(route.title(), route.description()).badge(count_badge))
                .child(filter_bar),
        )
    }
}

pub fn render_startup_page(params: StartupRouteParams<'_>) -> gpui::AnyElement {
    let mut page = StartupPage::new(
        params.entries,
        params.filter_state,
        params.search_query,
        params.search_focused,
        params.search_hovered,
        params.search_selection,
        params.open_menu_id,
        params.hovered_card_id,
    )
    .dropdown_state(params.dropdown_state)
    .filter_handlers(params.filter_handlers)
    .search_focus(params.search_focus);

    page.on_change_search = Some(params.on_change_search);
    page.on_hover_search = Some(params.on_hover_search);
    page.on_focus_search = Some(params.on_focus_search);
    page.on_selection_search = Some(params.on_selection_search);
    page.on_hover_card = Some(params.on_hover_card);
    page.on_toggle = Some(params.on_toggle);
    page.on_delete = Some(params.on_delete);
    page.on_open_folder = Some(params.on_open_folder);
    page.on_open_source = Some(params.on_open_source);
    page.on_copy_path = Some(params.on_copy_path);
    page.on_hover_tooltip = Some(params.on_hover_tooltip);
    page.on_toggle_menu = Some(params.on_toggle_menu);

    page.into_any_element()
}

#[cfg(test)]
mod tests;
