use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SpringAnimation,
    SpringConfig, StatefulInteractiveElement, Styled, Window, deferred, div, ease_in_out, img, px,
};

use crate::entities::startup::search::matches_startup_query;
use crate::entities::startup::{StartupEntry, StartupSource, StartupStatus};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;
use crate::shared::ui::icon::Icon;
use crate::shared::ui::search_input::{SearchChangeHandler, SearchInput};
use crate::shared::ui::smooth_scroll::SmoothVirtualList;
use crate::shared::ui::switch::Switch;
use crate::widgets::sidebar::lerp_rgba;

pub type StartupToggleHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupDeleteHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupActionHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;
pub type FilterSelectHandler = Arc<dyn Fn(Option<StartupSource>, &mut Window, &mut App) + 'static>;
pub type SearchHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchFocusHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectionHandler =
    Arc<dyn Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static>;
pub type StartupHoverCardHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
struct StartupCardHandlers {
    toggle: Option<StartupToggleHandler>,
    delete: Option<StartupDeleteHandler>,
    open_folder: Option<StartupActionHandler>,
    open_source: Option<StartupActionHandler>,
    copy_path: Option<StartupActionHandler>,
    hover_tt: Option<TooltipHoverHandler>,
    toggle_menu: Option<MenuToggleHandler>,
    hover_card: Option<StartupHoverCardHandler>,
    hovered_card_id: Option<String>,
}

#[derive(IntoElement)]
pub struct StartupPage {
    entries: Vec<StartupEntry>,
    active_filter: Option<StartupSource>,
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
    on_select_filter: Option<FilterSelectHandler>,
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
        active_filter: Option<StartupSource>,
        search_query: impl Into<String>,
        search_focused: bool,
        search_hovered: bool,
        search_selection: Option<(usize, usize)>,
        open_menu_id: Option<String>,
        hovered_card_id: Option<String>,
    ) -> Self {
        Self {
            entries,
            active_filter,
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
            on_select_filter: None,
            on_change_search: None,
            on_hover_search: None,
            on_focus_search: None,
            on_selection_search: None,
            on_hover_card: None,
        }
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

    #[must_use]
    pub fn on_select_filter(
        mut self,
        handler: impl Fn(Option<StartupSource>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_filter = Some(Arc::new(handler));
        self
    }
}

fn render_source_badge(
    entry: &StartupEntry,
    theme: &Theme,
    on_hover_tt: Option<TooltipHoverHandler>,
) -> impl IntoElement {
    let source_col = entry.source.color(theme);
    let tooltip_label = match entry.source {
        StartupSource::Registry => rust_i18n::t!("startup.source_registry").to_string(),
        StartupSource::StartupFolder => rust_i18n::t!("startup.source_folder").to_string(),
        StartupSource::Service => rust_i18n::t!("startup.source_service").to_string(),
        StartupSource::ScheduledTask => rust_i18n::t!("startup.source_task").to_string(),
    };

    let tt_h = on_hover_tt.clone();
    let tt_str = tooltip_label;

    div()
        .id(ElementId::Name(format!("{}_src_badge", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .rounded(px(4.0))
        .bg(source_col.opacity(0.12))
        .border_1()
        .border_color(source_col.opacity(0.3))
        .cursor_pointer()
        .on_mouse_move(move |event, window, cx| {
            if let Some(ref h) = tt_h {
                h(
                    Some(TooltipState {
                        text: tt_str.clone().into(),
                        cursor_pos: event.position,
                    }),
                    window,
                    cx,
                );
            }
        })
        .on_hover(move |hovered, window, cx| {
            if !hovered {
                if let Some(ref h) = on_hover_tt {
                    h(None, window, cx);
                }
            }
        })
        .child(
            Icon::new(entry.source.icon())
                .size(px(11.0))
                .color(source_col),
        )
}

fn render_scope_badge(
    entry: &StartupEntry,
    theme: &Theme,
    on_hover_tt: Option<TooltipHoverHandler>,
) -> impl IntoElement {
    let tooltip_label = match entry.scope {
        crate::entities::startup::StartupScope::CurrentUser => {
            rust_i18n::t!("startup.scope_current_user").to_string()
        }
        crate::entities::startup::StartupScope::AllUsers => {
            rust_i18n::t!("startup.scope_all_users").to_string()
        }
    };

    let tt_h = on_hover_tt.clone();
    let tt_str = tooltip_label;

    div()
        .id(ElementId::Name(format!("{}_scope_badge", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .rounded(px(4.0))
        .bg(theme.input_bg)
        .border_1()
        .border_color(theme.input_border)
        .cursor_pointer()
        .on_mouse_move(move |event, window, cx| {
            if let Some(ref h) = tt_h {
                h(
                    Some(TooltipState {
                        text: tt_str.clone().into(),
                        cursor_pos: event.position,
                    }),
                    window,
                    cx,
                );
            }
        })
        .on_hover(move |hovered, window, cx| {
            if !hovered {
                if let Some(ref h) = on_hover_tt {
                    h(None, window, cx);
                }
            }
        })
        .child(
            Icon::new(entry.scope.icon())
                .size(px(11.0))
                .color(theme.text_muted),
        )
}

#[allow(clippy::too_many_lines)]
fn render_action_menu(
    entry: &StartupEntry,
    theme: &Theme,
    handlers: &StartupCardHandlers,
) -> impl IntoElement {
    let entry_folder = entry.clone();
    let entry_src = entry.clone();
    let entry_copy = entry.clone();
    let entry_del = entry.clone();

    let on_folder = handlers.open_folder.clone();
    let on_source = handlers.open_source.clone();
    let on_copy = handlers.copy_path.clone();
    let on_del = handlers.delete.clone();
    let close_fn = handlers.toggle_menu.clone();

    let close_fn_click = close_fn.clone();
    let entry_id = entry.id.clone();

    let box_el = div()
        .id(ElementId::Name(format!("{}_menu_popover", entry.id).into()))
        .absolute()
        .top_full()
        .right_0()
        .mt(px(4.0))
        .w(px(210.0))
        .p(px(4.0))
        .rounded_lg()
        .bg(theme.card_bg)
        .border_1()
        .border_color(theme.card_border)
        .shadow_md()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .on_mouse_down_out(move |_, window, cx| {
            if let Some(ref h) = close_fn {
                h(None, window, cx);
            }
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(8.0))
                .py(px(6.0))
                .rounded_md()
                .hover(move |s| s.bg(theme.button_hover))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if let Some(ref h) = on_folder {
                        h(&entry_folder, window, cx);
                    }
                    if let Some(ref h) = close_fn_click {
                        h(None, window, cx);
                    }
                    cx.stop_propagation();
                })
                .child(
                    Icon::new("icons/folder-open.svg")
                        .size(px(14.0))
                        .color(theme.text_muted),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_primary)
                        .child(rust_i18n::t!("startup.action_open_folder").to_string()),
                ),
        );

    let close_fn_src = handlers.toggle_menu.clone();
    let box_el = box_el.child(
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded_md()
            .hover(move |s| s.bg(theme.button_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(ref h) = on_source {
                    h(&entry_src, window, cx);
                }
                if let Some(ref h) = close_fn_src {
                    h(None, window, cx);
                }
                cx.stop_propagation();
            })
            .child(
                Icon::new("icons/external-link.svg")
                    .size(px(14.0))
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_primary)
                    .child(rust_i18n::t!("startup.action_open_source").to_string()),
            ),
    );

    let close_fn_copy = handlers.toggle_menu.clone();
    let box_el = box_el.child(
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded_md()
            .hover(move |s| s.bg(theme.button_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(ref h) = on_copy {
                    h(&entry_copy, window, cx);
                }
                if let Some(ref h) = close_fn_copy {
                    h(None, window, cx);
                }
                cx.stop_propagation();
            })
            .child(
                Icon::new("icons/copy.svg")
                    .size(px(14.0))
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_primary)
                    .child(rust_i18n::t!("startup.action_copy_path").to_string()),
            ),
    );

    let close_fn_del = handlers.toggle_menu.clone();
    let del_color = theme.accent_red;
    let box_el = box_el.child(
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded_md()
            .hover(move |s| s.bg(theme.button_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(ref h) = on_del {
                    h(&entry_del, window, cx);
                }
                if let Some(ref h) = close_fn_del {
                    h(None, window, cx);
                }
                cx.stop_propagation();
            })
            .child(
                Icon::new("icons/trash-2.svg")
                    .size(px(14.0))
                    .color(del_color),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(del_color)
                    .child(rust_i18n::t!("startup.action_delete").to_string()),
            ),
    );

    box_el.with_animation(
        ElementId::Name(format!("{entry_id}_menu_enter").into()),
        Animation::new(Duration::from_millis(150)).with_easing(ease_in_out),
        move |menu, delta| {
            let offset_y = 2.0 + delta * 4.0;
            menu.opacity(delta).mt(px(offset_y))
        },
    )
}

#[allow(clippy::too_many_lines)]
fn render_startup_card(
    entry: &StartupEntry,
    theme: &Theme,
    is_menu_open: bool,
    handlers: &StartupCardHandlers,
) -> AnyElement {
    let source_col = entry.source.color(theme);
    let entry_id = entry.id.clone();
    let is_enabled = entry.status == StartupStatus::Enabled;
    let entry_toggle = entry.clone();

    let on_toggle_card = handlers.toggle.clone();
    let switch_el =
        Switch::new(format!("{}_sw", entry.id), is_enabled).on_toggle(move |_val, window, cx| {
            if let Some(ref h) = on_toggle_card {
                h(&entry_toggle, window, cx);
            }
        });

    let menu_toggle_id = entry_id.clone();
    let on_toggle_menu_btn = handlers.toggle_menu.clone();
    let menu_btn_bg = if is_menu_open {
        theme.button_selected
    } else {
        theme.card_bg
    };
    let menu_icon_col = if is_menu_open {
        theme.accent_blue
    } else {
        theme.text_muted
    };

    let menu_btn = div()
        .id(ElementId::Name(format!("{}_more_btn", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(28.0))
        .rounded_md()
        .bg(menu_btn_bg)
        .cursor_pointer()
        .hover(move |s| s.bg(theme.button_hover))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            if let Some(ref h) = on_toggle_menu_btn {
                if is_menu_open {
                    h(None, window, cx);
                } else {
                    h(Some(menu_toggle_id.clone()), window, cx);
                }
            }
            cx.stop_propagation();
        })
        .child(
            Icon::new("icons/ellipsis-vertical.svg")
                .size(px(16.0))
                .color(menu_icon_col),
        );

    let secondary_text = entry
        .publisher
        .clone()
        .unwrap_or_else(|| rust_i18n::t!("startup.unknown_publisher").to_string());

    let tt_h1 = handlers.hover_tt.clone();
    let tt_h2 = handlers.hover_tt.clone();

    let app_icon_el = if let Some(ref icon_file) = entry.icon_path {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(38.0))
            .rounded_lg()
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.input_border)
            .flex_none()
            .child(img(icon_file.clone()).size(px(24.0)).rounded(px(4.0)))
    } else {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(38.0))
            .rounded_lg()
            .bg(source_col.opacity(0.12))
            .flex_none()
            .child(
                Icon::new(entry.source.icon())
                    .size(px(20.0))
                    .color(source_col),
            )
    };

    let is_card_hovered = handlers
        .hovered_card_id
        .as_ref()
        .is_some_and(|id| id == &entry.id);
    let target = if is_card_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
        .to(target)
        .with_epsilon(0.005);
    let card_id = entry.id.clone();
    let on_hover_c = handlers.hover_card.clone();
    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let accent_blue = theme.accent_blue;

    div()
        .id(ElementId::Name(format!("{}_card", entry.id).into()))
        .relative()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .h(px(58.0))
        .px(px(16.0))
        .py(px(10.0))
        .rounded(px(10.0))
        .border_1()
        .on_hover(move |&hovered, window, cx| {
            if let Some(ref h) = on_hover_c {
                h(
                    if hovered { Some(card_id.clone()) } else { None },
                    window,
                    cx,
                );
            }
        })
        .with_spring(
            ElementId::Name(format!("{}_hover_spring", entry.id).into()),
            spring,
            move |card, val| {
                let t = val.clamp(0.0, 1.0);
                let border = lerp_rgba(card_border, accent_blue, t);
                let bg = lerp_rgba(card_bg, input_bg, t * 0.4);
                card.bg(bg).border_color(border)
            },
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .flex_1()
                .min_w(px(0.0))
                .child(app_icon_el)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text_primary)
                                        .text_ellipsis()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .child(entry.display_name.clone()),
                                )
                                .child(render_source_badge(entry, theme, tt_h1))
                                .child(render_scope_badge(entry, theme, tt_h2)),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .line_height(px(15.0))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(theme.text_muted)
                                .text_ellipsis()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(secondary_text),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .flex_none()
                .child(switch_el)
                .child(div().relative().child(menu_btn).when(is_menu_open, |this| {
                    this.child(
                        deferred(render_action_menu(entry, theme, handlers)).with_priority(100),
                    )
                })),
        )
        .into_any_element()
}

fn render_filter_pill(
    source: Option<StartupSource>,
    active_filter: Option<StartupSource>,
    theme: &Theme,
    on_select_filter: Option<FilterSelectHandler>,
) -> impl IntoElement {
    let is_selected = active_filter == source;
    let label = match source {
        None => rust_i18n::t!("startup.filter_all").to_string(),
        Some(StartupSource::Registry) => rust_i18n::t!("startup.filter_registry").to_string(),
        Some(StartupSource::StartupFolder) => rust_i18n::t!("startup.filter_folder").to_string(),
        Some(StartupSource::Service) => rust_i18n::t!("startup.filter_services").to_string(),
        Some(StartupSource::ScheduledTask) => rust_i18n::t!("startup.filter_tasks").to_string(),
    };

    let pill_id = match source {
        None => "filter_all",
        Some(StartupSource::Registry) => "filter_reg",
        Some(StartupSource::StartupFolder) => "filter_folder",
        Some(StartupSource::Service) => "filter_svc",
        Some(StartupSource::ScheduledTask) => "filter_tasks",
    };

    let bg = if is_selected {
        theme.button_selected
    } else {
        theme.input_bg
    };

    let text_col = if is_selected {
        theme.text_primary
    } else {
        theme.text_muted
    };

    div()
        .id(ElementId::Name(pill_id.into()))
        .flex()
        .items_center()
        .justify_center()
        .px(px(12.0))
        .py(px(6.0))
        .rounded_md()
        .bg(bg)
        .border_1()
        .border_color(theme.card_border)
        .hover(|s| s.bg(theme.button_hover))
        .cursor_pointer()
        .text_xs()
        .font_weight(if is_selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(text_col)
        .on_click(move |_, window, cx| {
            if let Some(ref h) = on_select_filter {
                h(source, window, cx);
            }
        })
        .child(label)
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
                if let Some(source) = self.active_filter {
                    if e.source != source {
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
            .width(px(220.0))
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

        let filter_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(search_input)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(render_filter_pill(
                        None,
                        self.active_filter,
                        &theme,
                        self.on_select_filter.clone(),
                    ))
                    .child(render_filter_pill(
                        Some(StartupSource::StartupFolder),
                        self.active_filter,
                        &theme,
                        self.on_select_filter.clone(),
                    ))
                    .child(render_filter_pill(
                        Some(StartupSource::ScheduledTask),
                        self.active_filter,
                        &theme,
                        self.on_select_filter.clone(),
                    ))
                    .child(render_filter_pill(
                        Some(StartupSource::Registry),
                        self.active_filter,
                        &theme,
                        self.on_select_filter.clone(),
                    ))
                    .child(render_filter_pill(
                        Some(StartupSource::Service),
                        self.active_filter,
                        &theme,
                        self.on_select_filter,
                    )),
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
