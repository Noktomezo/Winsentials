use std::sync::Arc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type ActionHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    destructive: bool,
    disabled: bool,
    on_click: Option<ActionHandler>,
}

impl MenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            destructive: false,
            disabled: false,
            on_click: None,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon_path: impl Into<SharedString>) -> Self {
        self.icon = Some(icon_path.into());
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
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for MenuItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);

        let (text_color, icon_color, hover_bg, active_bg) = if self.destructive {
            (
                theme.accent_red,
                theme.accent_red,
                theme.accent_red.opacity(0.15),
                theme.accent_red.opacity(0.25),
            )
        } else {
            (
                theme.text_primary,
                theme.text_muted,
                theme.button_hover,
                theme.accent_active_bg,
            )
        };

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded_md();

        if self.disabled {
            base = base.opacity(0.45);
        } else {
            base = base
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(active_bg));

            if let Some(on_click) = self.on_click {
                base = base.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    (on_click)(window, cx);
                });
            }
        }

        let icon_el = self
            .icon
            .map(|p| Icon::new(p).size(px(14.0)).color(icon_color));

        base.children(icon_el)
            .child(div().text_xs().text_color(text_color).child(self.label))
    }
}
