use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Transformation, Window, div, px, radians, svg,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type ClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Destructive,
    Outline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

fn spinner_angle(progress: f32) -> f32 {
    progress * std::f32::consts::TAU
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon_left: Option<SharedString>,
    icon_right: Option<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    selected: bool,
    disabled: bool,
    loading: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon_left: None,
            icon_right: None,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            selected: false,
            disabled: false,
            loading: false,
            tooltip: None,
            on_click: None,
        }
    }

    #[must_use]
    pub fn icon_left(mut self, icon_path: impl Into<SharedString>) -> Self {
        self.icon_left = Some(icon_path.into());
        self
    }

    #[must_use]
    pub fn icon_right(mut self, icon_path: impl Into<SharedString>) -> Self {
        self.icon_right = Some(icon_path.into());
        self
    }

    #[must_use]
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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
}

impl RenderOnce for Button {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);

        let (height, padding_x, text_size, icon_size, rounded_radius) = match self.size {
            ButtonSize::Sm => (px(28.0), px(10.0), px(12.0), px(13.0), px(5.0)),
            ButtonSize::Md => (px(34.0), px(14.0), px(13.0), px(15.0), px(6.0)),
            ButtonSize::Lg => (px(40.0), px(18.0), px(14.0), px(16.0), px(8.0)),
        };

        let (bg, hover_bg, active_bg, text_color, border_color) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent_blue,
                theme.accent_blue.opacity(0.88),
                theme.accent_blue.opacity(0.75),
                theme.selected_text,
                None,
            ),
            ButtonVariant::Secondary => (
                theme.input_bg,
                theme.button_hover,
                theme.button_active,
                theme.text_primary,
                Some(theme.card_border),
            ),
            ButtonVariant::Ghost => (
                gpui::rgba(0x0000_0000),
                theme.accent_hover_bg,
                theme.accent_active_bg,
                theme.text_primary,
                None,
            ),
            ButtonVariant::Destructive => (
                theme.accent_red.opacity(0.18),
                theme.accent_red.opacity(0.28),
                theme.accent_red.opacity(0.40),
                theme.accent_red,
                Some(theme.accent_red.opacity(0.35)),
            ),
            ButtonVariant::Outline => (
                gpui::rgba(0x0000_0000),
                theme.accent_hover_bg,
                theme.accent_active_bg,
                theme.text_primary,
                Some(theme.card_border),
            ),
        };

        let spinner_id = format!("{:?}_btn_spinner", self.id);

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .h(height)
            .px(padding_x)
            .rounded(rounded_radius)
            .bg(bg)
            .text_size(text_size)
            .font_weight(FontWeight::MEDIUM)
            .text_color(text_color);

        if let Some(border) = border_color {
            base = base.border_1().border_color(border);
        }

        if self.disabled {
            if !self.loading {
                base = base.opacity(0.45);
            }
        } else {
            base = base
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(active_bg));

            if let Some(on_click) = self.on_click {
                base = base.on_click(move |event, window, cx| {
                    (on_click)(event, window, cx);
                });
            }
        }

        let left_child = if self.loading {
            let spinner = svg()
                .path("icons/loader-circle.svg")
                .size(icon_size)
                .text_color(text_color);

            if cx.reduce_motion() {
                Some(spinner.into_any_element())
            } else {
                Some(
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
                        .into_any_element(),
                )
            }
        } else {
            self.icon_left.map(|p| {
                Icon::new(p)
                    .size(icon_size)
                    .color(text_color)
                    .into_any_element()
            })
        };

        let right_child = self.icon_right.map(|p| {
            Icon::new(p)
                .size(icon_size)
                .color(text_color)
                .into_any_element()
        });

        base.children(left_child)
            .child(self.label)
            .children(right_child)
    }
}
