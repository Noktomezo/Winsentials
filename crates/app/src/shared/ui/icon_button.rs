use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, SharedString, SpringAnimation,
    StatefulInteractiveElement, Styled, Transformation, Window, div, px, radians, svg,
};

use crate::shared::motion::lerp_item_bg;
use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type ClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type MouseDownHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type HoverHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

fn spinner_angle(progress: f32) -> f32 {
    progress * std::f32::consts::TAU
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let color = self.icon_color.unwrap_or_else(|| {
            if self.destructive {
                theme.accent_red.into()
            } else if self.selected {
                theme.accent_cyan.into()
            } else {
                theme.text_primary.into()
            }
        });
        let spinner_id = format!("{:?}_spinner", self.id);
        let spring_id = format!("{:?}_bg_spring", self.id);

        let rounded_radius = (self.button_size * 0.2).clamp(px(4.0), px(8.0));

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(self.button_size)
            .rounded(rounded_radius);

        if self.disabled {
            if !self.loading {
                base = base.opacity(0.45);
            }
        } else if self.spring.is_some() {
            base = base
                .cursor_pointer()
                .active(move |s| s.bg(theme.accent_active_bg));

            if let Some(on_hover) = self.on_hover {
                base = base.on_hover(move |&hov, window, cx| {
                    (on_hover)(hov, window, cx);
                });
            }
        } else if self.destructive {
            base = base
                .cursor_pointer()
                .hover(move |s| s.bg(theme.accent_red.opacity(0.18)))
                .active(move |s| s.bg(theme.accent_red.opacity(0.35)));
        } else if self.selected {
            base = base.cursor_pointer().bg(theme.accent_selected_bg);
        } else {
            base = base
                .cursor_pointer()
                .hover(move |s| s.bg(theme.accent_hover_bg).text_color(theme.accent_cyan))
                .active(move |s| s.bg(theme.accent_active_bg));
        }

        if !self.disabled {
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

        let icon = if self.loading {
            let spinner = svg()
                .path("icons/loader-circle.svg")
                .size(self.icon_size)
                .text_color(color);
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
                .color(color)
                .into_any_element()
        };

        let content = base.child(icon);

        if let Some((spring, accent)) = self.spring {
            content
                .with_spring(
                    ElementId::Name(spring_id.into()),
                    spring,
                    move |btn, val| {
                        let bg = lerp_item_bg(accent, val);
                        btn.bg(bg)
                    },
                )
                .into_any_element()
        } else {
            content.into_any_element()
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
