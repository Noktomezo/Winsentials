use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, SpringAnimation,
    StatefulInteractiveElement, Styled, Transformation, Window, div, px, radians, svg,
};

use crate::components::icon::Icon;
use crate::motion::{hover_spring, lerp_item_bg, lerp_rgba};
use crate::theme::Theme;

pub type ClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type MouseDownHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type HoverHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

fn spinner_angle(progress: f32) -> f32 {
    progress * std::f32::consts::TAU
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonVariant {
    #[default]
    Ghost,
    Outline,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(IntoElement)]
#[allow(dead_code)]
pub struct IconButton {
    id: ElementId,
    icon_path: SharedString,
    icon_size: Pixels,
    button_size: Pixels,
    icon_color: Option<Hsla>,
    variant: IconButtonVariant,
    selected: bool,
    destructive: bool,
    disabled: bool,
    loading: bool,
    tooltip: Option<SharedString>,
    spring: Option<(SpringAnimation<f32>, Rgba)>,
    on_click: Option<ClickHandler>,
    on_mouse_down: Option<MouseDownHandler>,
    on_hover: Option<HoverHandler>,
}

#[allow(dead_code)]
impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon_path: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon_path: icon_path.into(),
            icon_size: px(16.0),
            button_size: px(32.0),
            icon_color: None,
            variant: IconButtonVariant::Ghost,
            selected: false,
            destructive: false,
            disabled: false,
            loading: false,
            tooltip: None,
            spring: None,
            on_click: None,
            on_mouse_down: None,
            on_hover: None,
        }
    }

    #[must_use]
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn outline(mut self, outline: bool) -> Self {
        self.variant = if outline {
            IconButtonVariant::Outline
        } else {
            IconButtonVariant::Ghost
        };
        self
    }

    #[must_use]
    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
        self
    }

    #[must_use]
    pub fn button_size(mut self, size: Pixels) -> Self {
        self.button_size = size;
        self
    }

    #[must_use]
    pub fn icon_color(mut self, color: impl Into<Hsla>) -> Self {
        self.icon_color = Some(color.into());
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    #[must_use]
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_mouse_down(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_mouse_down = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn spring(mut self, spring: SpringAnimation<f32>, accent: Rgba) -> Self {
        self.spring = Some((spring, accent));
        self
    }

    #[must_use]
    pub fn on_hover(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    #[allow(clippy::too_many_lines)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let default_color: Hsla = if self.destructive {
            theme.accent_red.into()
        } else if self.selected {
            theme.accent_blue.into()
        } else {
            theme.text_primary.into()
        };
        let color = self.icon_color.unwrap_or(default_color);
        let spinner_id = format!("{:?}_spinner", self.id);
        let spring_id = format!("{:?}_bg_spring", self.id);

        let rounded_radius = (self.button_size * 0.2).clamp(px(4.0), px(8.0));

        let hover_id = ElementId::Name(format!("{:?}_hover_state", self.id).into());
        let hover_state = window.use_keyed_state(hover_id, cx, |_, _| false);
        let mut hovered = *hover_state.read(cx);

        if self.disabled && hovered {
            hover_state.update(cx, |state, _| *state = false);
            hovered = false;
        }

        let is_hovered = hovered && !self.disabled;
        let is_outline = self.variant == IconButtonVariant::Outline;

        let (bg_rest, bg_hover) = if self.destructive {
            (gpui::rgba(0x0000_0000), theme.accent_red.opacity(0.18))
        } else if self.selected {
            (theme.accent_selected_bg, theme.accent_selected_bg)
        } else if is_outline {
            (theme.input_bg, theme.button_hover)
        } else {
            (gpui::rgba(0x0000_0000), theme.accent_hover_bg)
        };

        let (text_color_rest, text_color_hover): (Rgba, Rgba) = if self.destructive {
            (theme.accent_red, theme.accent_red)
        } else if self.selected {
            (theme.accent_blue, theme.accent_blue)
        } else if self.icon_color.is_some() {
            (color.into(), color.into())
        } else {
            (theme.text_primary, theme.accent_blue)
        };

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(self.button_size)
            .rounded(rounded_radius);

        if is_outline {
            base = base.border_1().border_color(theme.card_border);
        }

        let hover_state_for_event = hover_state;
        let on_hover_cb = self.on_hover;
        let is_disabled = self.disabled;

        base = base.on_hover(move |&hov, window, cx| {
            let active_hov = hov && !is_disabled;
            hover_state_for_event.update(cx, |state, cx| {
                if *state != active_hov {
                    *state = active_hov;
                    cx.notify();
                }
            });
            if let Some(ref h) = on_hover_cb {
                h(hov, window, cx);
            }
        });

        if self.disabled {
            if !self.loading {
                base = base.opacity(0.45);
            }
        } else {
            base = base
                .cursor_pointer()
                .active(move |s| s.bg(theme.accent_active_bg));

            if let Some(on_click) = self.on_click {
                base = base.on_click(move |event, window, cx| {
                    (on_click)(event, window, cx);
                });
            }

            if let Some(on_mouse_down) = self.on_mouse_down {
                base = base.on_mouse_down(gpui::MouseButton::Left, move |_event, window, cx| {
                    cx.stop_propagation();
                    (on_mouse_down)(window, cx);
                });
            }
        }

        let current_icon_color: Hsla = if self.icon_color.is_some() {
            color
        } else if self.destructive {
            theme.accent_red.into()
        } else if self.selected || is_hovered {
            theme.accent_blue.into()
        } else {
            theme.text_primary.into()
        };

        let icon = if self.loading {
            let spinner = svg()
                .path(self.icon_path.clone())
                .size(self.icon_size)
                .text_color(current_icon_color);
            if cx.reduce_motion() {
                spinner.into_any_element()
            } else {
                spinner
                    .with_animation(
                        ElementId::Name(spinner_id.into()),
                        Animation::new(Duration::from_millis(850)).repeat(),
                        |icon, delta| {
                            icon.with_transformation(Transformation::rotate(radians(
                                spinner_angle(delta),
                            )))
                        },
                    )
                    .into_any_element()
            }
        } else {
            Icon::new(self.icon_path)
                .size(self.icon_size)
                .color(current_icon_color)
                .into_any_element()
        };

        let content = base.child(icon);

        if let Some((custom_spring, accent)) = self.spring {
            content
                .with_spring(
                    ElementId::Name(spring_id.into()),
                    custom_spring,
                    move |btn, val| {
                        let bg = lerp_item_bg(accent, val);
                        btn.bg(bg)
                    },
                )
                .into_any_element()
        } else if cx.reduce_motion() || self.disabled {
            let mut el = content
                .bg(if is_hovered { bg_hover } else { bg_rest })
                .text_color(if is_hovered {
                    text_color_hover
                } else {
                    text_color_rest
                });
            if is_outline {
                el = el.border_color(if is_hovered {
                    theme.accent_blue.opacity(0.5)
                } else {
                    theme.card_border
                });
            }
            el.into_any_element()
        } else {
            let spring = hover_spring(if is_hovered { 1.0 } else { 0.0 });
            let border_rest = theme.card_border;
            let border_hover = theme.accent_blue.opacity(0.5);
            content
                .with_spring(
                    ElementId::Name(spring_id.into()),
                    spring,
                    move |btn, val| {
                        let progress = val.clamp(0.0, 1.0);
                        let bg = lerp_rgba(bg_rest, bg_hover, progress);
                        let text_col = lerp_rgba(text_color_rest, text_color_hover, progress);
                        let mut el = btn.bg(bg).text_color(text_col);
                        if is_outline {
                            el = el.border_color(lerp_rgba(border_rest, border_hover, progress));
                        }
                        el
                    },
                )
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::spinner_angle;

    #[test]
    fn spinner_completes_one_turn() {
        assert!((spinner_angle(1.0) - std::f32::consts::TAU).abs() < f32::EPSILON);
    }
}
