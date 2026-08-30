use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::entities::startup::{StartupEntry, StartupSource, StartupStatus};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;
use crate::shared::ui::animated_grid::{VirtualGridConfig, render_virtual_animated_grid};
use crate::shared::ui::icon::Icon;
use crate::shared::ui::smooth_scroll::SmoothScroll;
use crate::shared::ui::switch::Switch;

pub type StartupToggleHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupDeleteHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type StartupActionHandler = Arc<dyn Fn(&StartupEntry, &mut Window, &mut App) + 'static>;
pub type TooltipHoverHandler = Arc<dyn Fn(Option<TooltipState>, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Arc<dyn Fn(Option<String>, &mut Window, &mut App) + 'static>;
pub type FilterSelectHandler = Arc<dyn Fn(Option<StartupSource>, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
struct StartupCardHandlers {
    toggle: Option<StartupToggleHandler>,
    delete: Option<StartupDeleteHandler>,
    open_folder: Option<StartupActionHandler>,
    open_source: Option<StartupActionHandler>,
    copy_path: Option<StartupActionHandler>,
    hover_tt: Option<TooltipHoverHandler>,
    toggle_menu: Option<MenuToggleHandler>,
}

#[derive(IntoElement)]
pub struct StartupPage {
    entries: Vec<StartupEntry>,
    active_filter: Option<StartupSource>,
    open_menu_id: Option<String>,
    sidebar_expanded: bool,
    on_toggle: Option<StartupToggleHandler>,
    on_delete: Option<StartupDeleteHandler>,
    on_open_folder: Option<StartupActionHandler>,
    on_open_source: Option<StartupActionHandler>,
    on_copy_path: Option<StartupActionHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
    on_toggle_menu: Option<MenuToggleHandler>,
    on_select_filter: Option<FilterSelectHandler>,
}

impl StartupPage {
    #[must_use]
    pub fn new(
        entries: Vec<StartupEntry>,
        active_filter: Option<StartupSource>,
        open_menu_id: Option<String>,
        sidebar_expanded: bool,
    ) -> Self {
        Self {
            entries,
            active_filter,
            open_menu_id,
            sidebar_expanded,
            on_toggle: None,
            on_delete: None,
            on_open_folder: None,
            on_open_source: None,
            on_copy_path: None,
            on_hover_tooltip: None,
            on_toggle_menu: None,
            on_select_filter: None,
        }
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
    tt_handler: Option<TooltipHoverHandler>,
) -> impl IntoElement {
    let source = entry.source;
    let label = source.label();
    let col = source.color(theme);

    div()
        .id(ElementId::Name(format!("{}_src_badge", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_hover(move |hovered, window, cx| {
            if let Some(ref h) = tt_handler {
                if *hovered {
                    let mouse_pos = window.mouse_position();
                    h(
                        Some(TooltipState {
                            text: label.clone().into(),
                            cursor_pos: mouse_pos,
                        }),
                        window,
                        cx,
                    );
                } else {
                    h(None, window, cx);
                }
            }
        })
        .child(Icon::new(source.icon()).size(px(14.0)).color(col))
}

fn render_scope_badge(
    entry: &StartupEntry,
    theme: &Theme,
    tt_handler: Option<TooltipHoverHandler>,
) -> impl IntoElement {
    let scope = entry.scope;
    let label = scope.label();
    let col = theme.text_muted;

    div()
        .id(ElementId::Name(format!("{}_scope_badge", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_hover(move |hovered, window, cx| {
            if let Some(ref h) = tt_handler {
                if *hovered {
                    let mouse_pos = window.mouse_position();
                    h(
                        Some(TooltipState {
                            text: label.clone().into(),
                            cursor_pos: mouse_pos,
                        }),
                        window,
                        cx,
                    );
                } else {
                    h(None, window, cx);
                }
            }
        })
        .child(Icon::new(scope.icon()).size(px(14.0)).color(col))
}

#[allow(clippy::too_many_arguments)]
fn render_menu_row(
    id: String,
    icon: &'static str,
    label: String,
    is_danger: bool,
    theme: &Theme,
    entry: StartupEntry,
    action: Option<StartupActionHandler>,
    toggle_menu: Option<MenuToggleHandler>,
) -> impl IntoElement {
    let text_col = if is_danger {
        theme.accent_red
    } else {
        theme.text_primary
    };
    let icon_col = if is_danger {
        theme.accent_red
    } else {
        theme.text_muted
    };
    let hover_bg = if is_danger {
        theme.accent_red.opacity(0.12)
    } else {
        theme.button_hover
    };

    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(8.0))
        .py(px(6.0))
        .rounded_md()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .text_xs()
        .text_color(text_col)
        .on_click(move |_, window, cx| {
            if let Some(ref h) = action {
                h(&entry, window, cx);
            }
            if let Some(ref close) = toggle_menu {
                close(None, window, cx);
            }
        })
        .child(Icon::new(icon).size(px(14.0)).color(icon_col))
        .child(label)
}

fn render_action_menu(
    entry: &StartupEntry,
    theme: &Theme,
    handlers: &StartupCardHandlers,
) -> impl IntoElement {
    let entry_id = entry.id.clone();
    let folder_row = render_menu_row(
        format!("{entry_id}_act_folder"),
        "icons/folder.svg",
        rust_i18n::t!("startup.action_open_folder").to_string(),
        false,
        theme,
        entry.clone(),
        handlers.open_folder.clone(),
        handlers.toggle_menu.clone(),
    );

    let copy_row = render_menu_row(
        format!("{entry_id}_act_copy"),
        "icons/copy.svg",
        rust_i18n::t!("startup.action_copy_path").to_string(),
        false,
        theme,
        entry.clone(),
        handlers.copy_path.clone(),
        handlers.toggle_menu.clone(),
    );

    let src_row = render_menu_row(
        format!("{entry_id}_act_src"),
        "icons/external-link.svg",
        rust_i18n::t!("startup.action_open_source").to_string(),
        false,
        theme,
        entry.clone(),
        handlers.open_source.clone(),
        handlers.toggle_menu.clone(),
    );

    let del_row = render_menu_row(
        format!("{entry_id}_act_del"),
        "icons/trash-2.svg",
        rust_i18n::t!("startup.action_delete").to_string(),
        true,
        theme,
        entry.clone(),
        handlers.delete.clone(),
        handlers.toggle_menu.clone(),
    );

    div()
        .absolute()
        .top(px(40.0))
        .right(px(0.0))
        .w(px(220.0))
        .bg(theme.card_bg)
        .border_1()
        .border_color(theme.card_border)
        .rounded_lg()
        .shadow_md()
        .p(px(4.0))
        .gap(px(2.0))
        .flex()
        .flex_col()
        .child(folder_row)
        .child(copy_row)
        .child(src_row)
        .child(div().h(px(1.0)).w_full().bg(theme.main_border).my(px(2.0)))
        .child(del_row)
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
    let menu_btn = div()
        .id(ElementId::Name(format!("{}_more_btn", entry.id).into()))
        .flex()
        .items_center()
        .justify_center()
        .size(px(28.0))
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(theme.button_hover))
        .on_click(move |_, window, cx| {
            if let Some(ref h) = on_toggle_menu_btn {
                if is_menu_open {
                    h(None, window, cx);
                } else {
                    h(Some(menu_toggle_id.clone()), window, cx);
                }
            }
        })
        .child(
            Icon::new("icons/ellipsis-vertical.svg")
                .size(px(16.0))
                .color(theme.text_muted),
        );

    let secondary_text = entry
        .publisher
        .clone()
        .unwrap_or_else(|| rust_i18n::t!("startup.unknown_publisher").to_string());

    let tt_h1 = handlers.hover_tt.clone();
    let tt_h2 = handlers.hover_tt.clone();

    div()
        .id(ElementId::Name(format!("{}_card", entry.id).into()))
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .h(px(68.0))
        .p(px(14.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(theme.card_border)
        .bg(theme.card_bg)
        .hover(|s| s.border_color(theme.accent_blue))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
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
                        ),
                )
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
                .child(menu_btn),
        )
        .when(is_menu_open, |this| {
            this.child(render_action_menu(entry, theme, handlers))
        })
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Startup;

        let total_count = self.entries.len();
        let enabled_count = self
            .entries
            .iter()
            .filter(|e| e.status == StartupStatus::Enabled)
            .count();

        let filtered_entries: Vec<StartupEntry> = self
            .entries
            .into_iter()
            .filter(|e| {
                if let Some(source) = self.active_filter {
                    e.source == source
                } else {
                    true
                }
            })
            .collect();

        let sidebar_w = if self.sidebar_expanded {
            px(200.0)
        } else {
            px(40.0)
        };
        let available_w = (window.viewport_size().width - sidebar_w - px(32.0)).max(px(320.0));

        let handlers = StartupCardHandlers {
            toggle: self.on_toggle,
            delete: self.on_delete,
            open_folder: self.on_open_folder,
            open_source: self.on_open_source,
            copy_path: self.on_copy_path,
            hover_tt: self.on_hover_tooltip,
            toggle_menu: self.on_toggle_menu,
        };

        let filter_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
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
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(format!(
                        "{}: {total_count} • {}: {enabled_count}",
                        rust_i18n::t!("startup.total"),
                        rust_i18n::t!("startup.enabled")
                    )),
            );

        let (scroll_y, viewport_h) = SmoothScroll::get_scroll_offset(route.id(), window, cx);

        let content_el = if filtered_entries.is_empty() {
            div()
                .flex()
                .items_center()
                .justify_center()
                .h(px(160.0))
                .w_full()
                .text_sm()
                .text_color(theme.text_muted)
                .child(rust_i18n::t!("startup.empty").to_string())
                .into_any_element()
        } else {
            let config = VirtualGridConfig::new(
                available_w,
                px(360.0),
                px(68.0),
                px(12.0),
                scroll_y,
                viewport_h,
            );
            render_virtual_animated_grid("startup_grid", config, &filtered_entries, |_i, entry| {
                let is_menu_open = self.open_menu_id.as_ref().is_some_and(|id| id == &entry.id);
                let elem = render_startup_card(entry, &theme, is_menu_open, &handlers);
                (entry.id.clone(), elem)
            })
            .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .child(PageHeader::new(route.title(), route.description()))
            .child(filter_bar)
            .child(content_el)
    }
}
