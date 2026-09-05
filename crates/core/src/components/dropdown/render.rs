use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled,
    Transformation, ease_in_out, px, radians, svg, div,
};

use crate::components::icon::Icon;
use crate::components::marquee_text::MarqueeText;
use crate::motion::{lerp_item_bg, lerp_item_text};
use crate::theme::Theme;

#[allow(clippy::wildcard_imports)]
use super::types::*;

pub(crate) struct DropdownMenuParams {
    pub items: Vec<DropdownItem>,
    pub selected_value: &'static str,
    pub hovered_opt: Option<&'static str>,
    pub is_open: bool,
    pub is_closing: bool,
    pub opens_upwards: bool,
    pub trigger_width: gpui::Pixels,
    pub dropdown_id_str: String,
    pub theme: Theme,
    pub on_select: Option<DropdownSelectHandler>,
    pub on_delete: Option<DropdownDeleteHandler>,
    pub on_hover_opt: Option<DropdownOptionHoverHandler>,
    pub on_close: Option<DropdownCloseHandler>,
}

pub(crate) fn render_dropdown_chevron(
    is_open: bool,
    is_closing: bool,
    dropdown_id_str: &str,
    theme: &Theme,
) -> AnyElement {
    if is_open {
        svg()
            .path("icons/chevron-down.svg")
            .size(px(14.0))
            .text_color(theme.accent_blue)
            .with_animation(
                ElementId::Name(format!("{dropdown_id_str}_chevron_open").into()),
                Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                move |svg_el, delta| {
                    let angle = delta * std::f32::consts::PI;
                    svg_el.with_transformation(Transformation::rotate(radians(angle)))
                },
            )
            .into_any_element()
    } else if is_closing {
        svg()
            .path("icons/chevron-down.svg")
            .size(px(14.0))
            .text_color(theme.text_muted)
            .with_animation(
                ElementId::Name(format!("{dropdown_id_str}_chevron_close").into()),
                Animation::new(Duration::from_millis(140)).with_easing(ease_in_out),
                move |svg_el, delta| {
                    let angle = (1.0 - delta) * std::f32::consts::PI;
                    svg_el.with_transformation(Transformation::rotate(radians(angle)))
                },
            )
            .into_any_element()
    } else {
        svg()
            .path("icons/chevron-down.svg")
            .size(px(14.0))
            .text_color(theme.text_muted)
            .into_any_element()
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn render_dropdown_menu(params: DropdownMenuParams) -> AnyElement {
    let theme = params.theme;
    let is_open = params.is_open;
    let is_closing = params.is_closing;
    let hovered_opt = params.hovered_opt;
    let on_select = params.on_select;
    let on_delete = params.on_delete;
    let on_close = params.on_close;
    let on_hover_opt = params.on_hover_opt;
    let selected_value = params.selected_value;
    let dropdown_id_str = params.dropdown_id_str.clone();
    let trigger_width = params.trigger_width;
    let opens_upwards = params.opens_upwards;

    let mut options_list = div().flex().flex_col().w_full();
    let total_items = params.items.len();

    for (idx, item) in params.items.into_iter().enumerate() {
        let val = item.value;
        let label = item.label;
        let opt_icon = item.icon;
        let is_deletable = item.deletable;

        let is_selected = val == selected_value;
        let is_opt_hovered = hovered_opt == Some(val);
        let select_handler = on_select.clone();
        let delete_handler = on_delete.clone();
        let opt_hover_handler = on_hover_opt.clone();

        let target_state: f32 = if is_selected {
            1.0
        } else if is_opt_hovered {
            0.5
        } else {
            0.0
        };

        let state_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(target_state)
            .with_epsilon(0.005);

        let opt_icon_space = if opt_icon.is_some() {
            px(22.0)
        } else {
            px(0.0)
        };

        let has_trailing = is_selected || is_deletable;
        let trailing_space = if has_trailing {
            px(14.0) + px(8.0)
        } else {
            px(0.0)
        };

        let max_opt_label_width =
            (trigger_width - px(20.0) - px(2.0) - trailing_space - opt_icon_space).max(px(30.0));

        let opt_id = format!("{dropdown_id_str}_opt_{val}");
        let is_opt_active = is_opt_hovered && !is_closing;
        let opt_marquee_id = format!("{dropdown_id_str}_opt_marquee_{val}");
        let accent_blue = theme.accent_blue;
        let theme_input_bg = theme.input_bg;
        let theme_accent_hover_bg = theme.accent_hover_bg;
        let theme_text_muted = theme.text_muted;
        let close_handler = on_close.clone();

        let mut option_row = div()
            .id(ElementId::Name(opt_id.clone().into()))
            .debug_selector({
                let id_clone = opt_id.clone();
                move || id_clone
            })
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .h(px(32.0))
            .w_full()
            .px(px(10.0))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });

        if total_items == 1 {
            option_row = option_row.rounded(px(5.0));
        } else if idx == 0 {
            option_row = option_row.rounded_t(px(5.0));
        } else if idx == total_items - 1 {
            option_row = option_row.rounded_b(px(5.0));
        }

        let row_with_spring = option_row
            .on_hover(move |&hov, window, cx| {
                if let Some(ref h) = opt_hover_handler {
                    h(val, &hov, window, cx);
                }
            })
            .with_spring(
                ElementId::Name(format!("{opt_id}_spring").into()),
                state_spring,
                move |btn, spring_val| {
                    let bg = if is_selected {
                        accent_blue
                    } else {
                        lerp_item_bg(accent_blue, spring_val)
                    };
                    let current_fade = if is_selected {
                        accent_blue
                    } else if spring_val > 0.0 {
                        crate::motion::lerp_rgba(theme_input_bg, accent_blue, spring_val)
                    } else {
                        theme_input_bg
                    };
                    let current_text_color = if is_selected {
                        theme.selected_text
                    } else {
                        lerp_item_text(&theme, spring_val)
                    };

                    let opt_icon_el = opt_icon
                        .map(|icon_path| render_dropdown_icon(icon_path, current_text_color));

                    let left_stack = div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .children(opt_icon_el)
                        .child(
                            MarqueeText::new(
                                opt_marquee_id.clone(),
                                label.clone(),
                                max_opt_label_width,
                            )
                            .debug_name(opt_marquee_id)
                            .font_size(px(13.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(current_text_color)
                            .fade_color(current_fade)
                            .fade_width(px(8.0))
                            .active(is_opt_active),
                        );

                    let mut sel_area = div()
                        .id(ElementId::Name(format!("{opt_id}_sel_area").into()))
                        .flex()
                        .items_center()
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(left_stack);

                    if is_selected {
                        let close_cb = close_handler.clone();
                        sel_area = sel_area.cursor_default().on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            if let Some(ref h) = close_cb {
                                h(window, cx);
                            }
                        });
                    } else {
                        let select_cb = select_handler.clone();
                        sel_area = sel_area.cursor_pointer().on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            if let Some(ref h) = select_cb {
                                h(val, window, cx);
                            }
                        });
                    }

                    let right_el = if is_selected {
                        Some(
                            div()
                                .id(ElementId::Name(format!("{opt_id}_right_el").into()))
                                .debug_selector({
                                    let id_clone = opt_id.clone();
                                    move || format!("{id_clone}_right_el")
                                })
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(14.0))
                                .child(
                                    Icon::new("icons/check.svg")
                                        .size(px(14.0))
                                        .color(current_text_color),
                                )
                                .into_any_element(),
                        )
                    } else if is_deletable {
                        let del_cb = delete_handler.clone();
                        Some(
                            div()
                                .id(ElementId::Name(format!("{opt_id}_del_btn").into()))
                                .debug_selector({
                                    let id_clone = opt_id.clone();
                                    move || format!("{id_clone}_del_btn")
                                })
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(14.0))
                                .rounded(px(3.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(theme_accent_hover_bg))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    if let Some(ref h) = del_cb {
                                        h(val, window, cx);
                                    }
                                })
                                .child(
                                    Icon::new("icons/x.svg")
                                        .size(px(12.0))
                                        .color(theme_text_muted),
                                )
                                .into_any_element(),
                        )
                    } else {
                        None
                    };

                    btn.bg(bg).child(sel_area).children(right_el)
                },
            );

        options_list = options_list.child(row_with_spring);
    }

    if is_open {
        let mut box_el = div()
            .id(ElementId::Name(format!("{dropdown_id_str}_menu_box").into()))
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_menu_box")
            })
            .absolute()
            .left_0()
            .w(trigger_width)
            .rounded(px(6.0))
            .overflow_hidden()
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.card_border)
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            });

        if let Some(ref close_fn) = on_close {
            let close_cb = close_fn.clone();
            box_el = box_el.on_mouse_down_out(move |_, window, cx| {
                close_cb(window, cx);
            });
        }

        if opens_upwards {
            box_el
                .bottom_full()
                .with_animation(
                    ElementId::Name(format!("{dropdown_id_str}_menu_open_up").into()),
                    Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                    move |menu, delta| {
                        let offset_y = 2.0 + delta * 3.0;
                        menu.opacity(delta).mb(px(offset_y))
                    },
                )
                .child(options_list)
                .into_any_element()
        } else {
            box_el
                .top_full()
                .with_animation(
                    ElementId::Name(format!("{dropdown_id_str}_menu_open_down").into()),
                    Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                    move |menu, delta| {
                        let offset_y = 2.0 + delta * 3.0;
                        menu.opacity(delta).mt(px(offset_y))
                    },
                )
                .child(options_list)
                .into_any_element()
        }
    } else {
        let box_el = div()
            .id(ElementId::Name(
                format!("{dropdown_id_str}_menu_box_close").into(),
            ))
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_menu_box_close")
            })
            .absolute()
            .left_0()
            .w(trigger_width)
            .rounded(px(6.0))
            .overflow_hidden()
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.card_border)
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            });

        if opens_upwards {
            box_el
                .bottom_full()
                .with_animation(
                    ElementId::Name(format!("{dropdown_id_str}_menu_close_up").into()),
                    Animation::new(Duration::from_millis(140)).with_easing(ease_in_out),
                    move |menu, delta| {
                        let offset_y = 5.0 - delta * 3.0;
                        menu.opacity(1.0 - delta).mb(px(offset_y))
                    },
                )
                .child(options_list)
                .into_any_element()
        } else {
            box_el
                .top_full()
                .with_animation(
                    ElementId::Name(format!("{dropdown_id_str}_menu_close_down").into()),
                    Animation::new(Duration::from_millis(140)).with_easing(ease_in_out),
                    move |menu, delta| {
                        let offset_y = 5.0 - delta * 3.0;
                        menu.opacity(1.0 - delta).mt(px(offset_y))
                    },
                )
                .child(options_list)
                .into_any_element()
        }
    }
}