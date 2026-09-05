use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled,
    Window, deferred, div, ease_in_out, px,
};

use crate::components::marquee_text::MarqueeText;
use crate::theme::Theme;

pub mod render;
pub mod types;

#[cfg(test)]
mod tests;

pub(crate) use render::*;
pub use types::*;

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
        let dropdown_id_str = format!("{:?}", self.id)
            .replace("Name(\"", "")
            .replace("\")", "")
            .replace('\"', "")
            .replace(' ', "_");
        let trigger_width = self.width.unwrap_or(px(150.0));

        let icon_el = self
            .icon
            .as_ref()
            .map(|icon_path| render_dropdown_icon(icon_path, theme.accent_blue));

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
            .child(render_dropdown_chevron(is_open, is_closing, &dropdown_id_str, &theme));

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

        let neutral_border = theme.card_border;
        let blue_border = theme.accent_blue;
        let hover_blue_border = theme.accent_hover_bg;

        let icon_space = if self.icon.is_some() {
            px(22.0)
        } else {
            px(0.0)
        };
        let max_label_width =
            (trigger_width - px(20.0) - px(2.0) - px(14.0) - px(8.0) - icon_space).max(px(30.0));

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
            let menu_content = render_dropdown_menu(DropdownMenuParams {
                items: self.items,
                selected_value,
                hovered_opt,
                is_open,
                is_closing,
                opens_upwards: self.upward,
                trigger_width,
                dropdown_id_str: dropdown_id_str.clone(),
                theme,
                on_select,
                on_delete,
                on_hover_opt,
                on_close,
            });
            root_container = root_container.child(deferred(menu_content).with_priority(100));
        }

        root_container
    }
}