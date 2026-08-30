use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig, StatefulInteractiveElement,
    Styled, Transformation, Window, deferred, div, ease_in_out, img, px, radians, svg,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::{lerp_item_bg, lerp_item_text};

pub type DropdownSelectHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownDeleteHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type DropdownToggleHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownCloseHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DropdownOptionHoverHandler = Arc<dyn Fn(&str, &bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownItem {
    pub value: &'static str,
    pub label: &'static str,
    pub icon: Option<&'static str>,
    pub deletable: bool,
}

impl DropdownItem {
    #[must_use]
    pub const fn new(value: &'static str, label: &'static str, icon: Option<&'static str>) -> Self {
        Self {
            value,
            label,
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
    closing: bool,
    morphing: bool,
    hovered: bool,
    upward: bool,
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
            closing: false,
            morphing: false,
            hovered: false,
            upward: false,
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
        img(icon_path.to_string())
            .w(px(16.0))
            .h(px(11.0))
            .rounded(px(2.0))
            .into_any_element()
    } else {
        Icon::new(icon_path.to_string())
            .size(px(14.0))
            .color(current_color)
            .into_any_element()
    }
}

impl RenderOnce for Dropdown {
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let is_open = self.open;
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
        let dropdown_id_str = format!("{:?}", self.id);

        let icon_el = self
            .icon
            .map(|icon_path| render_dropdown_icon(&icon_path, theme.accent_blue));

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

        // Pure fade morph animation for trigger icon & text ONLY when active selection change happens
        let base_left_stack = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .overflow_hidden()
            .children(icon_el)
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.text_primary)
                    .truncate()
                    .child(self.current_label),
            );

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
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .h(px(32.0))
            .w(px(150.0))
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
                        crate::widgets::sidebar::lerp_rgba(neutral_border, hover_blue_border, t)
                    } else {
                        let t = (v - 0.5) / 0.5;
                        crate::widgets::sidebar::lerp_rgba(hover_blue_border, blue_border, t)
                    };
                    el.border_color(color)
                },
            )
            .child(left_morph_stack)
            .child(chevron_el);

        let mut root_container = div()
            .id(ElementId::Name(format!("{dropdown_id_str}_root").into()))
            .relative()
            .w(px(150.0))
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

                // Target state: 0.0 (rest) -> 0.5 (hover: 50% cyan) -> 1.0 (selected: 100% solid cyan)
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

                let current_text_color = lerp_item_text(&theme, target_state);

                let opt_icon_el =
                    opt_icon.map(|icon_path| render_dropdown_icon(icon_path, current_text_color));

                let left_stack = div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .overflow_hidden()
                    .children(opt_icon_el)
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(if target_state >= 0.5 {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .text_color(current_text_color)
                            .truncate()
                            .child(label),
                    );

                let opt_id = format!("{dropdown_id_str}_opt_{val}");

                // Right element: 14px wide to match chevron alignment perfectly with 10px padding
                let right_el = if is_selected {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(14.0))
                        .h(px(18.0))
                        .child(
                            Icon::new("icons/check.svg")
                                .size(px(14.0))
                                .color(current_text_color),
                        )
                        .into_any_element()
                } else if is_deletable {
                    let del_cb = delete_handler;
                    div()
                        .id(ElementId::Name(format!("{opt_id}_del_btn").into()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(14.0))
                        .h(px(18.0))
                        .rounded(px(3.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent_hover_bg))
                        .on_click(move |_, window, cx| {
                            if let Some(ref h) = del_cb {
                                h(val, window, cx);
                            }
                        })
                        .child(
                            Icon::new("icons/x.svg")
                                .size(px(12.0))
                                .color(theme.text_muted),
                        )
                        .into_any_element()
                } else {
                    div().w(px(14.0)).h(px(18.0)).into_any_element()
                };

                let accent_blue = theme.accent_blue;
                let select_handler_cb = select_handler.clone();

                let mut option_row = div()
                    .id(ElementId::Name(opt_id.clone().into()))
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .w_full()
                    .px(px(10.0));

                if total_items == 1 {
                    option_row = option_row.rounded(px(5.0));
                } else if idx == 0 {
                    option_row = option_row.rounded_t(px(5.0));
                } else if idx == total_items - 1 {
                    option_row = option_row.rounded_b(px(5.0));
                }

                let mut sel_area = div()
                    .id(ElementId::Name(format!("{opt_id}_sel_area").into()))
                    .flex()
                    .items_center()
                    .flex_1()
                    .h_full()
                    .child(left_stack);

                if is_selected {
                    sel_area = sel_area.cursor_default();
                } else {
                    sel_area = sel_area.cursor_pointer().on_click(move |_, window, cx| {
                        if let Some(ref h) = select_handler_cb {
                            h(val, window, cx);
                        }
                    });
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
                        move |btn, val| {
                            let bg = lerp_item_bg(accent_blue, val);
                            btn.bg(bg)
                        },
                    )
                    .child(sel_area)
                    .child(right_el);

                options_list = options_list.child(row_with_spring);
            }

            let opens_upwards = self.upward;

            let menu_content = if is_open {
                let mut box_el = div()
                    .id(ElementId::Name(
                        format!("{dropdown_id_str}_menu_box").into(),
                    ))
                    .absolute()
                    .left_0()
                    .w(px(150.0))
                    .rounded(px(6.0))
                    .overflow_hidden()
                    .bg(theme.input_bg)
                    .border_1()
                    .border_color(theme.input_border)
                    .shadow_lg();

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
                    .absolute()
                    .left_0()
                    .w(px(150.0))
                    .rounded(px(6.0))
                    .overflow_hidden()
                    .bg(theme.input_bg)
                    .border_1()
                    .border_color(theme.input_border)
                    .shadow_lg();

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
