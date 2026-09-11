use std::sync::Arc;

use gpui::{App, Window};

use crate::entities::startup::{StartupEntry, StartupScope, StartupSource, StartupStatus};
use crate::shared::ui::TooltipState;

pub type StartupToggleHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupDeleteHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupActionHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;
pub type FilterSelectHandler = Arc<dyn Fn(Option<StartupSource>, &mut Window, &mut App) + 'static>;
pub type ScopeFilterSelectHandler =
    Arc<dyn Fn(Option<StartupScope>, &mut Window, &mut App) + 'static>;
pub type SourceFilterSelectHandler =
    Arc<dyn Fn(Option<StartupSource>, &mut Window, &mut App) + 'static>;
pub type StatusFilterSelectHandler =
    Arc<dyn Fn(Option<StartupStatus>, &mut Window, &mut App) + 'static>;
pub type VoidActionHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;
pub type DropdownOptionHoverHandler =
    Arc<dyn Fn(&'static str, &'static str, bool, &mut Window, &mut App) + 'static>;
pub type StartupSearchChangeHandler = Arc<dyn Fn(String, &mut Window, &mut App) + 'static>;
pub type SearchHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchFocusHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectionHandler =
    Arc<dyn Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static>;
pub type StartupHoverCardHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StartupFilterState {
    pub scope: Option<StartupScope>,
    pub source: Option<StartupSource>,
    pub status: Option<StartupStatus>,
    pub is_open: bool,
    pub is_closing: bool,
    pub reset_closing: bool,
}

impl StartupFilterState {
    #[must_use]
    pub const fn is_any_active(&self) -> bool {
        self.scope.is_some() || self.source.is_some() || self.status.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StartupDropdownState {
    pub open_dropdown: Option<&'static str>,
    pub open_dropdown_upward: bool,
    pub opening_dropdown: Option<&'static str>,
    pub closing_dropdown: Option<&'static str>,
    pub hovered_dropdown: Option<&'static str>,
    pub hovered_option: Option<(&'static str, &'static str)>,
}

#[allow(clippy::struct_excessive_bools)]
pub struct StartupRouteParams<'a> {
    pub entries: Vec<StartupEntry>,
    pub filter_state: StartupFilterState,
    pub dropdown_state: StartupDropdownState,
    pub search_query: &'a str,
    pub search_focused: bool,
    pub search_hovered: bool,
    pub search_selection: Option<(usize, usize)>,
    pub search_focus: &'a gpui::FocusHandle,
    pub open_menu_id: Option<String>,
    pub hovered_card_id: Option<String>,
    pub filter_handlers: super::filter::StartupFilterHandlers,
    pub on_change_search: StartupSearchChangeHandler,
    pub on_hover_search: SearchHoverHandler,
    pub on_focus_search: SearchFocusHandler,
    pub on_selection_search: SearchSelectionHandler,
    pub on_hover_card: StartupHoverCardHandler,
    pub on_toggle: StartupToggleHandler,
    pub on_delete: StartupDeleteHandler,
    pub on_open_folder: StartupActionHandler,
    pub on_open_source: StartupActionHandler,
    pub on_copy_path: StartupActionHandler,
    pub on_hover_tooltip: TooltipHoverHandler,
    pub on_toggle_menu: MenuToggleHandler,
}
