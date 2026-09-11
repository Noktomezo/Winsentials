use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, ElementId, IntoElement, ParentElement, Styled, div,
    ease_in_out, px,
};

use super::filter_dropdowns::{build_scope_dropdown, build_source_dropdown, build_status_dropdown};
use super::types::{
    DropdownHoverHandler, DropdownOptionHoverHandler, DropdownToggleHandler,
    ScopeFilterSelectHandler, SourceFilterSelectHandler, StartupDropdownState, StartupFilterState,
    StatusFilterSelectHandler, VoidActionHandler,
};
use crate::shared::ui::{IconButton, IconButtonVariant, SearchInput};

#[derive(Clone, Default)]
pub struct StartupFilterHandlers {
    pub on_toggle_filters: Option<VoidActionHandler>,
    pub on_select_scope: Option<ScopeFilterSelectHandler>,
    pub on_select_source: Option<SourceFilterSelectHandler>,
    pub on_select_status: Option<StatusFilterSelectHandler>,
    pub on_reset_filters: Option<VoidActionHandler>,
    pub on_toggle_dropdown: Option<DropdownToggleHandler>,
    pub on_hover_dropdown: Option<DropdownHoverHandler>,
    pub on_hover_option: Option<DropdownOptionHoverHandler>,
    pub on_close_dropdowns: Option<VoidActionHandler>,
}

pub(crate) fn render_filter_bar(
    search_input: SearchInput,
    filters: StartupFilterState,
    dropdowns: StartupDropdownState,
    handlers: &StartupFilterHandlers,
) -> impl IntoElement {
    let is_active = filters.is_any_active();
    let filter_icon = if is_active {
        "icons/filter-x.svg"
    } else {
        "icons/filter.svg"
    };

    let toggle_handler = handlers.on_toggle_filters.clone();
    let filter_button = IconButton::new("startup_filter_toggle", filter_icon)
        .variant(IconButtonVariant::Outline)
        .selected(filters.is_open)
        .tooltip(rust_i18n::t!("startup.filters_tooltip"))
        .on_click(move |_, window, cx| {
            if let Some(ref h) = toggle_handler {
                h(window, cx);
            }
        });

    let search_row = div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .w_full()
        .child(div().flex_1().w_full().child(search_input))
        .child(filter_button);

    let panel_element = render_filter_panel(filters, dropdowns, handlers);

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w_full()
        .child(search_row)
        .children(panel_element)
}

fn render_filter_panel(
    filters: StartupFilterState,
    dropdowns: StartupDropdownState,
    handlers: &StartupFilterHandlers,
) -> Option<AnyElement> {
    if !filters.is_open && !filters.is_closing {
        return None;
    }

    let scope_dd = build_scope_dropdown(filters, dropdowns, handlers);
    let source_dd = build_source_dropdown(filters, dropdowns, handlers);
    let status_dd = build_status_dropdown(filters, dropdowns, handlers);

    let reset_handler = handlers.on_reset_filters.clone();
    let reset_button = IconButton::new("startup_reset_filters", "icons/rotate-ccw.svg")
        .variant(IconButtonVariant::Ghost)
        .tooltip(rust_i18n::t!("startup.reset_filters"))
        .on_click(move |_, window, cx| {
            if let Some(ref h) = reset_handler {
                h(window, cx);
            }
        });

    let dropdowns_row = div()
        .flex()
        .flex_1()
        .min_w(px(0.0))
        .items_center()
        .gap(px(8.0))
        .child(div().flex_1().min_w(px(0.0)).child(scope_dd))
        .child(div().flex_1().min_w(px(0.0)).child(source_dd))
        .child(div().flex_1().min_w(px(0.0)).child(status_dd));

    let reset_element = if filters.reset_closing {
        Some(
            div()
                .w(px(0.0))
                .ml(px(0.0))
                .opacity(0.0)
                .flex()
                .items_center()
                .justify_center()
                .overflow_hidden()
                .flex_shrink_0()
                .with_animation(
                    ElementId::Name("startup_reset_button_exit".into()),
                    Animation::new(Duration::from_millis(150)).with_easing(ease_in_out),
                    move |el, delta| {
                        let progress = 1.0 - delta;
                        let width = progress * 32.0;
                        let ml = progress * 8.0;
                        el.w(px(width)).ml(px(ml)).opacity(progress)
                    },
                )
                .child(
                    div()
                        .size(px(32.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(reset_button),
                )
                .into_any_element(),
        )
    } else if filters.is_any_active() {
        Some(
            div()
                .w(px(32.0))
                .ml(px(8.0))
                .opacity(1.0)
                .flex()
                .items_center()
                .justify_center()
                .overflow_hidden()
                .flex_shrink_0()
                .with_animation(
                    ElementId::Name("startup_reset_button_enter".into()),
                    Animation::new(Duration::from_millis(180)).with_easing(ease_in_out),
                    move |el, delta| {
                        let width = delta * 32.0;
                        let ml = delta * 8.0;
                        el.w(px(width)).ml(px(ml)).opacity(delta)
                    },
                )
                .child(
                    div()
                        .size(px(32.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(reset_button),
                )
                .into_any_element(),
        )
    } else {
        None
    };

    let mut panel = div().flex().items_center().w_full().child(dropdowns_row);

    if let Some(reset_el) = reset_element {
        panel = panel.child(reset_el);
    }

    let animated_panel = if filters.is_closing {
        panel
            .with_animation(
                ElementId::Name("startup_filter_panel_exit".into()),
                Animation::new(Duration::from_millis(140)).with_easing(ease_in_out),
                move |el, delta| {
                    let offset_y = delta * -6.0;
                    el.opacity(1.0 - delta).mt(px(offset_y))
                },
            )
            .into_any_element()
    } else {
        panel
            .with_animation(
                ElementId::Name("startup_filter_panel_enter".into()),
                Animation::new(Duration::from_millis(180)).with_easing(ease_in_out),
                move |el, delta| {
                    let offset_y = (1.0 - delta) * -6.0;
                    el.opacity(delta).mt(px(offset_y))
                },
            )
            .into_any_element()
    };

    Some(animated_panel)
}
