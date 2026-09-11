use gpui::IntoElement;

use super::filter::StartupFilterHandlers;
use super::types::{StartupDropdownState, StartupFilterState};
use crate::entities::startup::{StartupScope, StartupSource, StartupStatus};
use crate::shared::ui::Dropdown;

pub(crate) fn build_scope_dropdown(
    filters: StartupFilterState,
    dropdowns: StartupDropdownState,
    handlers: &StartupFilterHandlers,
) -> impl IntoElement {
    let scope_val = match filters.scope {
        None => "any",
        Some(StartupScope::CurrentUser) => "current",
        Some(StartupScope::AllUsers) => "all",
    };
    let scope_title = rust_i18n::t!("startup.filter_scope_title");
    let scope_opt_label = match filters.scope {
        None => rust_i18n::t!("startup.filter_any"),
        Some(StartupScope::CurrentUser) => rust_i18n::t!("startup.filter_user_current"),
        Some(StartupScope::AllUsers) => rust_i18n::t!("startup.filter_user_all"),
    };
    let scope_label = format!("{scope_title}: {scope_opt_label}");
    let scope_icon = match filters.scope {
        Some(s) => s.icon(),
        None => "icons/users.svg",
    };

    let scope_select = handlers.on_select_scope.clone();
    let scope_toggle = handlers.on_toggle_dropdown.clone();
    let scope_hover = handlers.on_hover_dropdown.clone();
    let scope_hover_opt = handlers.on_hover_option.clone();
    let scope_close = handlers.on_close_dropdowns.clone();

    Dropdown::new("startup_scope", scope_label, scope_val)
        .icon(scope_icon)
        .w_full()
        .localized_options(vec![
            (
                "any",
                rust_i18n::t!("startup.filter_any").into(),
                Some("icons/users.svg"),
            ),
            (
                "current",
                rust_i18n::t!("startup.filter_user_current").into(),
                Some("icons/user.svg"),
            ),
            (
                "all",
                rust_i18n::t!("startup.filter_user_all").into(),
                Some("icons/users.svg"),
            ),
        ])
        .open(dropdowns.open_dropdown == Some("startup_scope"))
        .opening(dropdowns.opening_dropdown == Some("startup_scope"))
        .closing(dropdowns.closing_dropdown == Some("startup_scope"))
        .upward(dropdowns.open_dropdown_upward)
        .hovered(dropdowns.hovered_dropdown == Some("startup_scope"))
        .hovered_option(
            if let Some(("startup_scope", opt)) = dropdowns.hovered_option {
                Some(opt)
            } else {
                None
            },
        )
        .on_toggle(move |window, cx| {
            if let Some(ref h) = scope_toggle {
                h("startup_scope", window, cx);
            }
        })
        .on_select(move |val, window, cx| {
            if let Some(ref h) = scope_select {
                let scope = match val {
                    "current" => Some(StartupScope::CurrentUser),
                    "all" => Some(StartupScope::AllUsers),
                    _ => None,
                };
                h(scope, window, cx);
            }
        })
        .on_close(move |window, cx| {
            if let Some(ref h) = scope_close {
                h(window, cx);
            }
        })
        .on_hover_trigger(move |hov, window, cx| {
            if let Some(ref h) = scope_hover {
                h("startup_scope", *hov, window, cx);
            }
        })
        .on_hover_option(move |opt, &hov, window, cx| {
            if let Some(ref h) = scope_hover_opt {
                let static_opt = match opt {
                    "current" => "current",
                    "all" => "all",
                    _ => "any",
                };
                h("startup_scope", static_opt, hov, window, cx);
            }
        })
}

pub(crate) fn build_source_dropdown(
    filters: StartupFilterState,
    dropdowns: StartupDropdownState,
    handlers: &StartupFilterHandlers,
) -> impl IntoElement {
    let source_val = match filters.source {
        None => "any",
        Some(StartupSource::StartupFolder) => "folder",
        Some(StartupSource::Registry) => "registry",
        Some(StartupSource::Service) => "service",
        Some(StartupSource::ScheduledTask) => "task",
    };
    let source_title = rust_i18n::t!("startup.filter_source_title");
    let source_opt_label = match filters.source {
        None => rust_i18n::t!("startup.filter_any"),
        Some(StartupSource::StartupFolder) => rust_i18n::t!("startup.filter_folder"),
        Some(StartupSource::Registry) => rust_i18n::t!("startup.filter_registry"),
        Some(StartupSource::Service) => rust_i18n::t!("startup.filter_services"),
        Some(StartupSource::ScheduledTask) => rust_i18n::t!("startup.filter_tasks"),
    };
    let source_label = format!("{source_title}: {source_opt_label}");
    let source_icon = match filters.source {
        Some(s) => s.icon(),
        None => "icons/layers.svg",
    };

    let source_select = handlers.on_select_source.clone();
    let source_toggle = handlers.on_toggle_dropdown.clone();
    let source_hover = handlers.on_hover_dropdown.clone();
    let source_hover_opt = handlers.on_hover_option.clone();
    let source_close = handlers.on_close_dropdowns.clone();

    Dropdown::new("startup_source", source_label, source_val)
        .icon(source_icon)
        .w_full()
        .localized_options(vec![
            (
                "any",
                rust_i18n::t!("startup.filter_any").into(),
                Some("icons/layers.svg"),
            ),
            (
                "folder",
                rust_i18n::t!("startup.filter_folder").into(),
                Some("icons/folder.svg"),
            ),
            (
                "registry",
                rust_i18n::t!("startup.filter_registry").into(),
                Some("icons/binary.svg"),
            ),
            (
                "service",
                rust_i18n::t!("startup.filter_services").into(),
                Some("icons/cog.svg"),
            ),
            (
                "task",
                rust_i18n::t!("startup.filter_tasks").into(),
                Some("icons/clock.svg"),
            ),
        ])
        .open(dropdowns.open_dropdown == Some("startup_source"))
        .opening(dropdowns.opening_dropdown == Some("startup_source"))
        .closing(dropdowns.closing_dropdown == Some("startup_source"))
        .upward(dropdowns.open_dropdown_upward)
        .hovered(dropdowns.hovered_dropdown == Some("startup_source"))
        .hovered_option(
            if let Some(("startup_source", opt)) = dropdowns.hovered_option {
                Some(opt)
            } else {
                None
            },
        )
        .on_toggle(move |window, cx| {
            if let Some(ref h) = source_toggle {
                h("startup_source", window, cx);
            }
        })
        .on_select(move |val, window, cx| {
            if let Some(ref h) = source_select {
                let source = match val {
                    "folder" => Some(StartupSource::StartupFolder),
                    "registry" => Some(StartupSource::Registry),
                    "service" => Some(StartupSource::Service),
                    "task" => Some(StartupSource::ScheduledTask),
                    _ => None,
                };
                h(source, window, cx);
            }
        })
        .on_close(move |window, cx| {
            if let Some(ref h) = source_close {
                h(window, cx);
            }
        })
        .on_hover_trigger(move |hov, window, cx| {
            if let Some(ref h) = source_hover {
                h("startup_source", *hov, window, cx);
            }
        })
        .on_hover_option(move |opt, &hov, window, cx| {
            if let Some(ref h) = source_hover_opt {
                let static_opt = match opt {
                    "folder" => "folder",
                    "registry" => "registry",
                    "service" => "service",
                    "task" => "task",
                    _ => "any",
                };
                h("startup_source", static_opt, hov, window, cx);
            }
        })
}

pub(crate) fn build_status_dropdown(
    filters: StartupFilterState,
    dropdowns: StartupDropdownState,
    handlers: &StartupFilterHandlers,
) -> impl IntoElement {
    let status_val = match filters.status {
        None => "any",
        Some(StartupStatus::Enabled) => "enabled",
        Some(StartupStatus::Disabled) => "disabled",
    };
    let status_title = rust_i18n::t!("startup.filter_status_title");
    let status_opt_label = match filters.status {
        None => rust_i18n::t!("startup.filter_any"),
        Some(StartupStatus::Enabled) => rust_i18n::t!("startup.filter_enabled"),
        Some(StartupStatus::Disabled) => rust_i18n::t!("startup.filter_disabled"),
    };
    let status_label = format!("{status_title}: {status_opt_label}");
    let status_icon = match filters.status {
        None => "icons/circle.svg",
        Some(StartupStatus::Enabled) => "icons/circle-check.svg",
        Some(StartupStatus::Disabled) => "icons/circle-x.svg",
    };

    let status_select = handlers.on_select_status.clone();
    let status_toggle = handlers.on_toggle_dropdown.clone();
    let status_hover = handlers.on_hover_dropdown.clone();
    let status_hover_opt = handlers.on_hover_option.clone();
    let status_close = handlers.on_close_dropdowns.clone();

    Dropdown::new("startup_status", status_label, status_val)
        .icon(status_icon)
        .w_full()
        .localized_options(vec![
            (
                "any",
                rust_i18n::t!("startup.filter_any").into(),
                Some("icons/circle.svg"),
            ),
            (
                "enabled",
                rust_i18n::t!("startup.filter_enabled").into(),
                Some("icons/circle-check.svg"),
            ),
            (
                "disabled",
                rust_i18n::t!("startup.filter_disabled").into(),
                Some("icons/circle-x.svg"),
            ),
        ])
        .open(dropdowns.open_dropdown == Some("startup_status"))
        .opening(dropdowns.opening_dropdown == Some("startup_status"))
        .closing(dropdowns.closing_dropdown == Some("startup_status"))
        .upward(dropdowns.open_dropdown_upward)
        .hovered(dropdowns.hovered_dropdown == Some("startup_status"))
        .hovered_option(
            if let Some(("startup_status", opt)) = dropdowns.hovered_option {
                Some(opt)
            } else {
                None
            },
        )
        .on_toggle(move |window, cx| {
            if let Some(ref h) = status_toggle {
                h("startup_status", window, cx);
            }
        })
        .on_select(move |val, window, cx| {
            if let Some(ref h) = status_select {
                let status = match val {
                    "enabled" => Some(StartupStatus::Enabled),
                    "disabled" => Some(StartupStatus::Disabled),
                    _ => None,
                };
                h(status, window, cx);
            }
        })
        .on_close(move |window, cx| {
            if let Some(ref h) = status_close {
                h(window, cx);
            }
        })
        .on_hover_trigger(move |hov, window, cx| {
            if let Some(ref h) = status_hover {
                h("startup_status", *hov, window, cx);
            }
        })
        .on_hover_option(move |opt, &hov, window, cx| {
            if let Some(ref h) = status_hover_opt {
                let static_opt = match opt {
                    "enabled" => "enabled",
                    "disabled" => "disabled",
                    _ => "any",
                };
                h("startup_status", static_opt, hov, window, cx);
            }
        })
}
