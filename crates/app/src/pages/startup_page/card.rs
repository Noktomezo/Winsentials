use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, AnyElement, ElementId, FontWeight,
    InteractiveElement, IntoElement, ParentElement, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, deferred, div, ease_in_out, img, px,
};

use crate::entities::startup::{StartupEntry, StartupSource, StartupStatus};
use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;
use crate::shared::ui::switch::Switch;
use crate::shared::ui::{IconButton, MenuItem, TooltipState};
use crate::widgets::sidebar::lerp_rgba;

use super::types::*;
pub(crate) struct StartupCardHandlers {
    pub(crate) toggle: Option<StartupToggleHandler>,
    pub(crate) delete: Option<StartupDeleteHandler>,
    pub(crate) open_folder: Option<StartupActionHandler>,
    pub(crate) open_source: Option<StartupActionHandler>,
    pub(crate) copy_path: Option<StartupActionHandler>,
    pub(crate) hover_tt: Option<TooltipHoverHandler>,
    pub(crate) toggle_menu: Option<MenuToggleHandler>,
    pub(crate) hover_card: Option<StartupHoverCardHandler>,
    pub(crate) hovered_card_id: Option<String>,
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
        .size(px(16.0))
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
                .size(px(12.0))
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
        .size(px(16.0))
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
                .size(px(12.0))
                .color(theme.accent_blue),
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
    let close_fn_src = close_fn.clone();
    let close_fn_copy = close_fn.clone();
    let close_fn_del = close_fn.clone();
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
            MenuItem::new(
                "menu_open_folder",
                rust_i18n::t!("startup.action_open_folder").to_string(),
            )
            .icon("icons/folder-open.svg")
            .on_click(move |window, cx| {
                if let Some(ref h) = on_folder {
                    h(&entry_folder, window, cx);
                }
                if let Some(ref h) = close_fn_click {
                    h(None, window, cx);
                }
            }),
        )
        .child(
            MenuItem::new(
                "menu_open_src",
                rust_i18n::t!("startup.action_open_source").to_string(),
            )
            .icon("icons/external-link.svg")
            .on_click(move |window, cx| {
                if let Some(ref h) = on_source {
                    h(&entry_src, window, cx);
                }
                if let Some(ref h) = close_fn_src {
                    h(None, window, cx);
                }
            }),
        )
        .child(
            MenuItem::new(
                "menu_copy_path",
                rust_i18n::t!("startup.action_copy_path").to_string(),
            )
            .icon("icons/copy.svg")
            .on_click(move |window, cx| {
                if let Some(ref h) = on_copy {
                    h(&entry_copy, window, cx);
                }
                if let Some(ref h) = close_fn_copy {
                    h(None, window, cx);
                }
            }),
        )
        .child(
            MenuItem::new(
                "menu_delete",
                rust_i18n::t!("startup.action_delete").to_string(),
            )
            .icon("icons/trash-2.svg")
            .destructive(true)
            .on_click(move |window, cx| {
                if let Some(ref h) = on_del {
                    h(&entry_del, window, cx);
                }
                if let Some(ref h) = close_fn_del {
                    h(None, window, cx);
                }
            }),
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

pub(crate) const fn fallback_app_icon() -> &'static str {
    "icons/app-window.svg"
}

#[allow(clippy::too_many_lines)]
pub(crate) fn render_startup_card(
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
    let menu_icon_col = if is_menu_open {
        theme.accent_blue
    } else {
        theme.text_muted
    };

    let menu_btn = IconButton::new(
        ElementId::Name(format!("{}_more_btn", entry.id).into()),
        "icons/ellipsis-vertical.svg",
    )
    .selected(is_menu_open)
    .icon_color(menu_icon_col)
    .on_click(move |_, window, cx| {
        if let Some(ref h) = on_toggle_menu_btn {
            if is_menu_open {
                h(None, window, cx);
            } else {
                h(Some(menu_toggle_id.clone()), window, cx);
            }
        }
        cx.stop_propagation();
    });

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
            .size(px(32.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.card_border)
            .flex_none()
            .child(img(icon_file.clone()).size(px(16.0)).rounded(px(4.0)))
    } else {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(32.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.card_border)
            .flex_none()
            .child(
                Icon::new(fallback_app_icon())
                    .size(px(16.0))
                    .color(source_col),
            )
    };

    let is_card_hovered = handlers
        .hovered_card_id
        .as_ref()
        .is_some_and(|id| id == &entry.id);
    let target = if is_card_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
        .to(target)
        .with_epsilon(0.01);
    let card_id = entry.id.clone();
    let on_hover_c = handlers.hover_card.clone();
    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let input_border = theme.input_border;

    div()
        .id(ElementId::Name(format!("{}_card", entry.id).into()))
        .relative()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(10.0))
        .h(px(64.0))
        .p(px(16.0))
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
                let border = lerp_rgba(card_border, input_border, t);
                let bg = lerp_rgba(card_bg, input_bg, t);
                card.bg(bg).border_color(border)
            },
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .flex_1()
                .min_w(px(0.0))
                .child(app_icon_el)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .h(px(32.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .line_height(px(16.0))
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
                                .line_height(px(14.0))
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

