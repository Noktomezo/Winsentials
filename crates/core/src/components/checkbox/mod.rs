use std::sync::Arc;

use gpui::{
    AnimationExt, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Transformation, Window, div,
    px, size, svg,
};

use crate::motion::{hover_spring, lerp_rgba};
use crate::theme::Theme;

pub type CheckboxToggleHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// Production-grade, library-quality Checkbox component with smooth spring animations.
///
/// Features:
/// - Selection spring: Smooth background fill and centered glyph scaling pop on check/uncheck
/// - Hover spring: Continuous border lighting and subtle background brightening
/// - Indeterminate state support (represented by a minus icon)
/// - Active press feedback and disabled state styling
/// - Accessibility and reduced motion support
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    on_toggle: Option<CheckboxToggleHandler>,
}

impl Checkbox {
    #[must_use]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            indeterminate: false,
            disabled: false,
            on_toggle: None,
        }
    }

    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    #[must_use]
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id = self.id;
        let checked = self.checked;
        let indeterminate = self.indeterminate;
        let disabled = self.disabled;
        let on_toggle = self.on_toggle;
        let reduce_motion = cx.reduce_motion();

        let id_str = match &id {
            ElementId::Name(name) => name.to_string(),
            _ => format!("{id:?}"),
        };

        let hover_id = ElementId::Name(format!("{id_str}_hover").into());
        let hover_state = window.use_keyed_state(hover_id, cx, |_, _| false);
        let mut hovered = *hover_state.read(cx);
        if disabled && hovered {
            hover_state.update(cx, |s, _| *s = false);
            hovered = false;
        }

        let is_active = checked || indeterminate;
        let icon_path = if indeterminate {
            "icons/minus.svg"
        } else {
            "icons/check.svg"
        };

        let click_action = {
            let on_toggle = on_toggle.clone();
            move |window: &mut Window, cx: &mut App| {
                if let Some(ref h) = on_toggle {
                    let next = !checked;
                    h(next, window, cx);
                }
            }
        };

        let hover_state_for_event = hover_state;
        let mut container = div()
            .id(id)
            .debug_selector({
                let id_sel = id_str.clone();
                move || id_sel.clone()
            })
            .relative()
            .size(px(18.0))
            .rounded(px(5.0))
            .border_1()
            .overflow_hidden();

        if disabled {
            container = container.opacity(0.45);
        } else {
            container = container
                .cursor_pointer()
                .active(|s| s.opacity(0.85))
                .on_hover(move |&hov, _window, cx| {
                    hover_state_for_event.update(cx, |state, cx| {
                        if *state != hov {
                            *state = hov;
                            cx.notify();
                        }
                    });
                })
                .on_click(move |_event, window, cx| {
                    cx.stop_propagation();
                    click_action(window, cx);
                });
        }

        if reduce_motion {
            if is_active {
                let active_bg = if hovered && !disabled {
                    theme.accent_blue.opacity(0.88)
                } else {
                    theme.accent_blue
                };
                container
                    .bg(active_bg)
                    .border_color(active_bg)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(icon_path)
                            .size(px(12.0))
                            .text_color(theme.selected_text),
                    )
                    .into_any_element()
            } else {
                let border_col = if hovered && !disabled {
                    theme.accent_blue.opacity(0.65)
                } else {
                    theme.card_border
                };
                let bg_col = if hovered && !disabled {
                    theme.button_hover
                } else {
                    theme.input_bg
                };
                container
                    .bg(bg_col)
                    .border_color(border_col)
                    .into_any_element()
            }
        } else {
            let target_hover_val: f32 = if hovered && !disabled { 1.0 } else { 0.0 };
            let hover_anim = hover_spring(target_hover_val);
            let hover_anim_id = ElementId::Name(format!("{id_str}_hov_spring").into());

            let normal_border = theme.card_border;
            let hover_border = theme.accent_blue.opacity(0.65);
            let normal_bg = theme.input_bg;
            let hover_bg = theme.button_hover;

            let container_with_hover =
                container.with_spring(hover_anim_id, hover_anim, move |el, hover_val| {
                    let h = hover_val.clamp(0.0, 1.0);
                    let border_col = lerp_rgba(normal_border, hover_border, h);
                    let bg_col = lerp_rgba(normal_bg, hover_bg, h);
                    el.border_color(border_col).bg(bg_col)
                });

            let target_check_val: f32 = if is_active { 1.0 } else { 0.0 };
            let check_anim = SpringAnimation::new(SpringConfig::new(380.0, 30.0, 1.0))
                .to(target_check_val)
                .with_epsilon(0.005);
            let check_anim_id = ElementId::Name(format!("{id_str}_check_spring").into());

            let active_bg = if hovered && !disabled {
                theme.accent_blue.opacity(0.88)
            } else {
                theme.accent_blue
            };
            let text_color = theme.selected_text;

            let overlay_sel = format!("{id_str}_overlay");
            let check_overlay = div()
                .id(ElementId::Name(overlay_sel.clone().into()))
                .debug_selector(move || overlay_sel.clone())
                .absolute()
                .top(-px(1.0))
                .left(-px(1.0))
                .size(px(18.0))
                .rounded(px(5.0))
                .border_1()
                .flex()
                .items_center()
                .justify_center()
                .with_spring(check_anim_id, check_anim, move |layer, check_val| {
                    let c = check_val.clamp(0.0, 1.0);
                    if c <= 0.005 {
                        return layer.opacity(0.0);
                    }

                    let scale = (0.4 + 0.6 * c).clamp(0.0, 1.0);
                    let icon_opacity = c;

                    layer
                        .bg(active_bg)
                        .border_color(active_bg)
                        .opacity(c)
                        .child(
                            svg()
                                .path(icon_path)
                                .size(px(12.0))
                                .text_color(text_color.opacity(icon_opacity))
                                .with_transformation(Transformation::scale(size(scale, scale))),
                        )
                });

            container_with_hover.child(check_overlay).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests;
