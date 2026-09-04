use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Transformation, Window, deferred, div, ease_in_out, img,
    px, radians, svg,
};

use crate::components::icon::Icon;
use crate::components::marquee_text::MarqueeText;
use crate::motion::{lerp_item_bg, lerp_item_text};
use crate::theme::Theme;

pub type DropdownSelectHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownDeleteHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownCloseHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DropdownOptionHoverHandler = Arc<dyn Fn(&str, &bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownItem {
    pub value: &'static str,
    pub label: SharedString,
    pub icon: Option<&'static str>,
    pub deletable: bool,
}

impl DropdownItem {
    #[must_use]
    pub fn new(
        value: &'static str,
        label: impl Into<SharedString>,
        icon: Option<&'static str>,
    ) -> Self {
        Self {
            value,
            label: label.into(),
            icon,
            deletable: false,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn deletable(mut self, deletable: bool) -> Self {
        self.deletable = deletable;
        self
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(IntoElement)]
pub struct Dropdown {
    id: ElementId,
    icon: Option<SharedString>,
    current_label: SharedString,
    items: Vec<DropdownItem>,
    selected_value: &'static str,
    open: bool,
    opening: bool,
    closing: bool,
    morphing: bool,
    hovered: bool,
    upward: bool,
    width: Option<gpui::Pixels>,
    hovered_option: Option<&'static str>,
    on_toggle: Option<DropdownToggleHandler>,
    on_select: Option<DropdownSelectHandler>,
    on_delete: Option<DropdownDeleteHandler>,
    on_close: Option<DropdownCloseHandler>,
    on_hover_trigger: Option<DropdownHoverHandler>,
    on_hover_option: Option<DropdownOptionHoverHandler>,
}

impl Dropdown {
    #[must_use]
    pub fn new(
        id: impl Into<ElementId>,
        current_label: impl Into<SharedString>,
        selected_value: &'static str,
    ) -> Self {
        Self {
            id: id.into(),
            icon: None,
            current_label: current_label.into(),
            items: Vec::new(),
            selected_value,
            open: false,
            opening: false,
            closing: false,
            morphing: false,
            hovered: false,
            upward: false,
            width: None,
            hovered_option: None,
            on_toggle: None,
            on_select: None,
            on_delete: None,
            on_close: None,
            on_hover_trigger: None,
            on_hover_option: None,
        }
    }

    #[must_use]
    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn options(
        mut self,
        options: Vec<(&'static str, &'static str, Option<&'static str>)>,
    ) -> Self {
        self.items = options
            .into_iter()
            .map(|(val, lbl, ico)| DropdownItem::new(val, lbl, ico))
            .collect();
        self
    }

    #[must_use]
    pub fn localized_options(
        mut self,
        options: Vec<(&'static str, SharedString, Option<&'static str>)>,
    ) -> Self {
        self.items = options
            .into_iter()
            .map(|(value, label, icon)| DropdownItem::new(value, label, icon))
            .collect();
        self
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn items(mut self, items: Vec<DropdownItem>) -> Self {
        self.items = items;
        self
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub fn opening(mut self, opening: bool) -> Self {
        self.opening = opening;
        self
    }

    #[must_use]
    pub fn closing(mut self, closing: bool) -> Self {
        self.closing = closing;
        self
    }

    #[must_use]
    pub fn morphing(mut self, morphing: bool) -> Self {
        self.morphing = morphing;
        self
    }

    #[must_use]
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[must_use]
    pub fn upward(mut self, upward: bool) -> Self {
        self.upward = upward;
        self
    }

    #[must_use]
    pub fn hovered_option(mut self, hovered_option: Option<&'static str>) -> Self {
        self.hovered_option = hovered_option;
        self
    }

    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_select(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn on_delete(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_delete = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_trigger(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_trigger = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_option(
        mut self,
        handler: impl Fn(&str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_option = Some(Arc::new(handler));
        self
    }
}

#[must_use]
pub fn render_dropdown_icon(icon_path: &str, current_color: Rgba) -> gpui::AnyElement {
    if Path::new(icon_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
    {
        div()
            .flex_none()
            .child(
                img(icon_path.to_string())
                    .w(px(16.0))
                    .h(px(11.0))
                    .rounded(px(2.0)),
            )
            .into_any_element()
    } else {
        div()
            .flex_none()
            .child(
                Icon::new(icon_path.to_string())
                    .size(px(14.0))
                    .color(current_color),
            )
            .into_any_element()
    }
}

impl RenderOnce for Dropdown {
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let is_open = self.open;
        let _is_opening = self.opening;
        let is_closing = self.closing;
        let is_morphing = self.morphing;
        let is_hovered = self.hovered;
        let hovered_opt = self.hovered_option;
        let on_toggle = self.on_toggle.clone();
        let on_select = self.on_select.clone();
        let on_delete = self.on_delete.clone();
        let on_close = self.on_close;
        let on_hover = self.on_hover_trigger;
        let on_hover_opt = self.on_hover_option;
        let selected_value = self.selected_value;
        let dropdown_id_str = format!("{:?}", self.id)
            .replace("Name(\"", "")
            .replace("\")", "")
            .replace('"', "")
            .replace(' ', "_");
        let trigger_width = self.width.unwrap_or(px(150.0));

        let icon_el = self
            .icon
            .as_ref()
            .map(|icon_path| render_dropdown_icon(icon_path, theme.accent_blue));

        // Chevron rotation animation on open and close
        let chevron_el = if is_open {
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
        };

        let chevron_box = div()
            .id(ElementId::Name(
                format!("{dropdown_id_str}_chevron_box").into(),
            ))
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_chevron_box")
            })
            .flex_none()
            .size(px(14.0))
            .flex()
            .items_center()
            .justify_center()
            .child(chevron_el);

        // Spring-driven trigger border animation
        let trigger_target: f32 = if is_open {
            1.0
        } else if is_hovered {
            0.5
        } else {
            0.0
        };

        let trigger_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(trigger_target)
            .with_epsilon(0.005);

        let neutral_border = theme.input_border;
        let blue_border = theme.accent_blue;
        let hover_blue_border = theme.accent_hover_bg;

        let icon_space = if self.icon.is_some() {
            px(22.0)
        } else {
            px(0.0)
        };
        let max_label_width =
            (trigger_width - px(20.0) - px(2.0) - px(14.0) - px(8.0) - icon_space).max(px(30.0));

        // Pure fade morph animation for trigger icon & text ONLY when active selection change happens
        let base_left_stack = div()
            .id(ElementId::Name(
                format!("{dropdown_id_str}_label_stack").into(),
            ))
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_label_stack")
            })
            .flex()
            .items_center()
            .gap(px(8.0))
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .children(icon_el)
            .child({
                let is_trigger_marquee_active = is_hovered && !is_open && !is_closing;
                let trigger_marquee_id = format!("{dropdown_id_str}_trigger_marquee");
                MarqueeText::new(
                    trigger_marquee_id.clone(),
                    self.current_label.clone(),
                    max_label_width,
                )
                .debug_name(trigger_marquee_id)
                .font_size(px(13.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .fade_color(theme.input_bg)
                .fade_width(px(8.0))
                .active(is_trigger_marquee_active)
            });

        let left_morph_stack = if is_morphing {
            let label_anim_id = format!("{dropdown_id_str}_fade_morph_{selected_value}");
            base_left_stack
                .with_animation(
                    ElementId::Name(label_anim_id.into()),
                    Animation::new(Duration::from_millis(180)).with_easing(ease_in_out),
                    gpui::Styled::opacity,
                )
                .into_any_element()
        } else {
            base_left_stack.into_any_element()
        };

        let trigger = div()
            .id(self.id)
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_trigger")
            })
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .h(px(32.0))
            .w(trigger_width)
            .px(px(10.0))
            .rounded(px(6.0))
            .border_1()
            .bg(theme.input_bg)
            .cursor_pointer()
            .on_hover(move |&hovered, window, cx| {
                if let Some(ref h) = on_hover {
                    h(&hovered, window, cx);
                }
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                if let Some(ref h) = on_toggle {
                    h(window, cx);
                }
            })
            .with_spring(
                ElementId::Name(format!("{dropdown_id_str}_trigger_spring").into()),
                trigger_spring,
                move |el, val| {
                    let v = val.clamp(0.0, 1.0);
                    let color = if v <= 0.5 {
                        let t = v / 0.5;
                        crate::motion::lerp_rgba(neutral_border, hover_blue_border, t)
                    } else {
                        let t = (v - 0.5) / 0.5;
                        crate::motion::lerp_rgba(hover_blue_border, blue_border, t)
                    };
                    el.border_color(color)
                },
            )
            .child(left_morph_stack)
            .child(chevron_box);

        let mut root_container = div()
            .id(ElementId::Name(format!("{dropdown_id_str}_root").into()))
            .debug_selector({
                let id_clone = dropdown_id_str.clone();
                move || format!("{id_clone}_root")
            })
            .relative()
            .w(trigger_width)
            .child(trigger);

        if is_open || is_closing {
            // Options stack without vertical or inter-item gaps
            let mut options_list = div().flex().flex_col().w_full();
            let total_items = self.items.len();

            for (idx, item) in self.items.into_iter().enumerate() {
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
                    (trigger_width - px(20.0) - px(2.0) - trailing_space - opt_icon_space)
                        .max(px(30.0));

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

                            let opt_icon_el = opt_icon.map(|icon_path| {
                                render_dropdown_icon(icon_path, current_text_color)
                            });

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
                                sel_area =
                                    sel_area.cursor_default().on_click(move |_, window, cx| {
                                        cx.stop_propagation();
                                        if let Some(ref h) = close_cb {
                                            h(window, cx);
                                        }
                                    });
                            } else {
                                let select_cb = select_handler.clone();
                                sel_area =
                                    sel_area.cursor_pointer().on_click(move |_, window, cx| {
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

            let opens_upwards = self.upward;

            let menu_content = if is_open {
                let mut box_el = div()
                    .id(ElementId::Name(
                        format!("{dropdown_id_str}_menu_box").into(),
                    ))
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
                    .border_color(theme.input_border)
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

                // on_mouse_down_out on menu_box ensures clicking outside dismisses without intercepting option clicks inside!
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
                                let offset_y = 2.0 + delta * 3.0; // 2px -> 5px slide up
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
                                let offset_y = 2.0 + delta * 3.0; // 2px -> 5px slide down
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
                    .border_color(theme.input_border)
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
                                let offset_y = 5.0 - delta * 3.0; // 5px -> 2px slide down
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
                                let offset_y = 5.0 - delta * 3.0; // 5px -> 2px slide up
                                menu.opacity(1.0 - delta).mt(px(offset_y))
                            },
                        )
                        .child(options_list)
                        .into_any_element()
                }
            };

            // Deferred rendering ensures the popup paints AFTER all ancestor cards, borders, and sibling rows
            root_container = root_container.child(deferred(menu_content).with_priority(100));
        }

        root_container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, VisualTestContext, size};

    struct TestDropdownView {
        open: bool,
        opening: bool,
        closing: bool,
        current_label: SharedString,
        width: Option<gpui::Pixels>,
    }

    impl Render for TestDropdownView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let mut dd = Dropdown::new("test_dd", self.current_label.clone(), "standard")
                .icon("icons/shield-check.svg")
                .options(vec![
                    ("standard", "Стандарт", Some("icons/shield-check.svg")),
                    ("mild", "Мягкий", Some("icons/feather.svg")),
                    (
                        "aggressive",
                        "Очень длинный агрессивный пресет с большим переполнением",
                        Some("icons/flame.svg"),
                    ),
                ])
                .open(self.open)
                .opening(self.opening)
                .closing(self.closing);

            if let Some(w) = self.width {
                dd = dd.width(w);
            }

            div().size_full().p(px(20.0)).child(dd)
        }
    }

    #[gpui::test]
    fn dropdown_chevron_and_trigger_maintain_geometry_under_various_languages(
        cx: &mut TestAppContext,
    ) {
        // Test labels across Russian, English, and extra long labels
        let test_cases = [
            // Russian CTF presets
            ("Стандарт", px(150.0)),
            ("Мягкий", px(150.0)),
            ("Агрессивный", px(150.0)),
            // English CTF presets
            ("Standard", px(150.0)),
            ("Mild", px(150.0)),
            ("Aggressive", px(150.0)),
            // Russian Keyboard repeat presets
            ("Сверхбыстрый", px(150.0)),
            // Long edge-case text to verify flex_none prevents chevron shrink
            (
                "ОченьДлинныйТекстДляПроверкиОверфлоуБезСжатияШеврона",
                px(150.0),
            ),
            // Custom wider dropdown
            ("Wide Preset Mode", px(180.0)),
        ];

        for (label, expected_width) in test_cases {
            let window = cx.open_window(size(px(600.0), px(400.0)), move |_, _| TestDropdownView {
                open: false,
                opening: false,
                closing: false,
                current_label: label.into(),
                width: if expected_width != px(150.0) {
                    Some(expected_width)
                } else {
                    None
                },
            });
            let mut cx = VisualTestContext::from_window(window.into(), cx);

            let trigger_bounds = cx
                .debug_bounds("test_dd_trigger")
                .expect("trigger must be rendered");
            let chevron_bounds = cx
                .debug_bounds("test_dd_chevron_box")
                .expect("chevron must be rendered");

            // 1. Trigger maintains exact expected width
            assert_eq!(trigger_bounds.size.width, expected_width);

            // 2. Chevron is NEVER squished or shrunk below 14x14px regardless of text length
            assert_eq!(chevron_bounds.size.width, px(14.0));
            assert_eq!(chevron_bounds.size.height, px(14.0));

            // 3. Chevron stays strictly inside trigger bounds (no overflow)
            assert!(chevron_bounds.right() <= trigger_bounds.right());
            assert!(chevron_bounds.left() >= trigger_bounds.left());
        }
    }

    #[gpui::test]
    fn dropdown_menu_box_matches_width_and_options_do_not_overflow(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "Стандарт".into(),
            width: None,
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_bounds = cx.debug_bounds("test_dd_trigger").unwrap();
        let menu_bounds = cx.debug_bounds("test_dd_menu_box").unwrap();

        // 1. Menu box width matches trigger width
        assert_eq!(menu_bounds.size.width, trigger_bounds.size.width);

        // 2. All options fit strictly inside menu box without horizontal overflow
        for (val, opt_selector) in [
            ("standard", "test_dd_opt_standard"),
            ("mild", "test_dd_opt_mild"),
            ("aggressive", "test_dd_opt_aggressive"),
        ] {
            let opt_bounds = cx
                .debug_bounds(opt_selector)
                .unwrap_or_else(|| panic!("option {val} must be rendered"));

            assert!(opt_bounds.left() >= menu_bounds.left());
            assert!(opt_bounds.right() <= menu_bounds.right());
        }

        // 3. Selected checkmark element is strictly 14px and doesn't squish
        let right_el_bounds = cx.debug_bounds("test_dd_opt_standard_right_el").unwrap();
        assert_eq!(right_el_bounds.size.width, px(14.0));
        assert_eq!(right_el_bounds.size.height, px(14.0));
    }

    #[gpui::test]
    fn dropdown_trigger_with_long_label_renders_marquee_with_fog(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: false,
            opening: false,
            closing: false,
            current_label: "Очень длинное название аудиоустройства или пресета переключения клавиш"
                .into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_bounds = cx.debug_bounds("test_dd_trigger").unwrap();
        let chevron_bounds = cx.debug_bounds("test_dd_chevron_box").unwrap();

        assert_eq!(trigger_bounds.size.width, px(150.0));
        assert_eq!(chevron_bounds.size.width, px(14.0));
        assert!(chevron_bounds.right() <= trigger_bounds.right());
    }

    #[gpui::test]
    fn dropdown_open_options_with_overflow_render_fog_correctly(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "Стандарт".into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let menu_bounds = cx.debug_bounds("test_dd_menu_box").unwrap();
        let selected_opt = cx.debug_bounds("test_dd_opt_standard").unwrap();
        let checkmark = cx.debug_bounds("test_dd_opt_standard_right_el").unwrap();

        assert_eq!(menu_bounds.size.width, px(150.0));
        assert!(selected_opt.left() >= menu_bounds.left());
        assert!(selected_opt.right() <= menu_bounds.right());
        assert_eq!(checkmark.size.width, px(14.0));
        assert!(checkmark.right() <= selected_opt.right());
    }

    #[gpui::test]
    fn dropdown_trigger_fog_remains_present_when_menu_is_open(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: false,
            closing: false,
            current_label: "Очень длинное название аудиоустройства или пресета переключения клавиш"
                .into(),
            width: Some(px(150.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let trigger_fog = cx
            .debug_bounds("test_dd_trigger_marquee_fade_right")
            .expect("trigger fog must remain visible even when menu is open");
        assert!(trigger_fog.size.width > px(0.0));
    }

    #[gpui::test]
    fn dropdown_animation_opening_and_closing_maintains_continuous_bounds(cx: &mut TestAppContext) {
        // 1. Opening phase: menu is marked open & opening
        let window = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: true,
            opening: true,
            closing: false,
            current_label: "Стандарт".into(),
            width: Some(px(160.0)),
        });
        let mut cx = VisualTestContext::from_window(window.into(), cx);

        let menu_opening_bounds = cx
            .debug_bounds("test_dd_menu_box")
            .expect("menu box must exist during open animation");
        assert_eq!(menu_opening_bounds.size.width, px(160.0));

        let opt_marquee = cx
            .debug_bounds("test_dd_opt_marquee_aggressive_anchor")
            .expect("overflowing option marquee anchor must be rendered");
        assert!(opt_marquee.size.width > px(0.0));

        let opt_fog_opening = cx
            .debug_bounds("test_dd_opt_marquee_aggressive_fade_right")
            .expect("overflowing option fog must be rendered during open animation");
        assert!(opt_fog_opening.size.width > px(0.0));

        // 2. Closing phase: open is false, closing is true
        let window_close = cx.open_window(size(px(600.0), px(400.0)), |_, _| TestDropdownView {
            open: false,
            opening: false,
            closing: true,
            current_label: "Стандарт".into(),
            width: Some(px(160.0)),
        });
        let mut cx_close = VisualTestContext::from_window(window_close.into(), &cx);

        let menu_closing_bounds = cx_close
            .debug_bounds("test_dd_menu_box_close")
            .expect("menu box close must exist during close animation");
        assert_eq!(menu_closing_bounds.size.width, px(160.0));

        // Marquee anchor preserves identical width and coordinates during closing
        let opt_marquee_close = cx_close
            .debug_bounds("test_dd_opt_marquee_aggressive_anchor")
            .expect("overflowing option marquee anchor must remain stable during close animation");
        assert_eq!(opt_marquee_close.size.width, opt_marquee.size.width);

        let opt_fog_closing = cx_close
            .debug_bounds("test_dd_opt_marquee_aggressive_fade_right")
            .expect("overflowing option fog must be rendered during close animation");
        assert!(opt_fog_closing.size.width > px(0.0));
    }
}
