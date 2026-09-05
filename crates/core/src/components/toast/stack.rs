use std::sync::Arc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, StatefulInteractiveElement, MouseButton, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, px,
};

use super::item::ToastItemView;
#[allow(clippy::wildcard_imports)]
use super::types::*;
#[derive(IntoElement)]
pub struct ToastStack {
    toasts: Vec<ToastData>,
    closing_id: Option<SharedString>,
    hovered_toast_btn: Option<(SharedString, usize)>,
    is_expanded: bool,
    position: ToastPosition,
    on_dismiss: Option<ToastDismissIdHandler>,
    on_hover_button: Option<ToastHoverBtnIdHandler>,
    on_hover_stack: Option<ToastStackHoverHandler>,
}

impl ToastStack {
    #[must_use]
    pub fn new(toasts: Vec<ToastData>) -> Self {
        let position = toasts
            .first()
            .map_or(ToastPosition::BottomRight, |t| t.position);
        Self {
            toasts,
            closing_id: None,
            hovered_toast_btn: None,
            is_expanded: false,
            position,
            on_dismiss: None,
            on_hover_button: None,
            on_hover_stack: None,
        }
    }

    #[must_use]
    pub fn closing_id(mut self, id: Option<SharedString>) -> Self {
        self.closing_id = id;
        self
    }

    #[must_use]
    pub fn hovered_toast_button(mut self, info: Option<(SharedString, usize)>) -> Self {
        self.hovered_toast_btn = info;
        self
    }

    #[must_use]
    pub const fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_button(
        mut self,
        handler: impl Fn(&str, usize, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_button = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_stack(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_stack = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ToastStack {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.toasts.is_empty() {
            return div().into_any_element();
        }

        let is_expanded = self.is_expanded;
        let on_dismiss = self.on_dismiss;
        let on_hover_btn = self.on_hover_button;
        let on_hover_stack = self.on_hover_stack;
        let closing_id = self.closing_id;
        let hovered_toast_btn = self.hovered_toast_btn;

        let total_count = self.toasts.len();
        let mut stack_container = div()
            .id(ElementId::Name("toast_stack_container".into()))
            .flex()
            .flex_col()
            .gap(if is_expanded { px(10.0) } else { px(0.0) });

        if total_count > 1 {
            stack_container = stack_container.on_hover(move |&hov, window, cx| {
                if let Some(ref h) = on_hover_stack {
                    h(&hov, window, cx);
                }
            });
        }

        stack_container = stack_container
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_up(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            });

        if total_count == 1 {
            let toast = self.toasts.into_iter().next().unwrap();
            let t_id = toast.id.clone();
            let is_closing = closing_id.as_ref() == Some(&t_id);
            let hov_btn = hovered_toast_btn
                .as_ref()
                .filter(|(id, _)| id == &t_id)
                .map(|(_, idx)| *idx);

            let on_dismiss_cb = on_dismiss;
            let on_hov_cb = on_hover_btn;
            let t_id_str: &'static str = Box::leak(t_id.to_string().into_boxed_str());

            let item_el = ToastItemView::new(toast)
                .closing(is_closing)
                .hovered_button(hov_btn)
                .on_dismiss(move |window, cx| {
                    if let Some(ref h) = on_dismiss_cb {
                        h(t_id_str, window, cx);
                    }
                })
                .on_hover_button(move |idx, is_hov, window, cx| {
                    if let Some(ref h) = on_hov_cb {
                        h(t_id_str, idx, is_hov, window, cx);
                    }
                });

            stack_container = stack_container.child(item_el);
        } else if is_expanded {
            for toast in self.toasts {
                let t_id = toast.id.clone();
                let is_closing = closing_id.as_ref() == Some(&t_id);
                let hov_btn = hovered_toast_btn
                    .as_ref()
                    .filter(|(id, _)| id == &t_id)
                    .map(|(_, idx)| *idx);

                let on_dismiss_cb = on_dismiss.clone();
                let on_hov_cb = on_hover_btn.clone();
                let t_id_str: &'static str = Box::leak(t_id.to_string().into_boxed_str());

                let item_el = ToastItemView::new(toast)
                    .closing(is_closing)
                    .hovered_button(hov_btn)
                    .on_dismiss(move |window, cx| {
                        if let Some(ref h) = on_dismiss_cb {
                            h(t_id_str, window, cx);
                        }
                    })
                    .on_hover_button(move |idx, is_hov, window, cx| {
                        if let Some(ref h) = on_hov_cb {
                            h(t_id_str, idx, is_hov, window, cx);
                        }
                    });

                stack_container = stack_container.child(item_el);
            }
        } else {
            // Render up to 3 toasts with Sonner stack offset
            let visible_toasts: Vec<_> = self.toasts.into_iter().rev().take(3).collect();
            let mut overlay_box = div()
                .id(ElementId::Name("toast_overlay_box".into()))
                .relative()
                .w(px(340.0))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(|_, _, cx| {
                    cx.stop_propagation();
                });

            for (depth, toast) in visible_toasts.into_iter().enumerate() {
                let t_id = toast.id.clone();
                let is_closing = closing_id.as_ref() == Some(&t_id);
                let hov_btn = hovered_toast_btn
                    .as_ref()
                    .filter(|(id, _)| id == &t_id)
                    .map(|(_, idx)| *idx);

                let on_dismiss_cb = on_dismiss.clone();
                let on_hov_cb = on_hover_btn.clone();
                let t_id_str: &'static str = Box::leak(t_id.to_string().into_boxed_str());

                let item_el = ToastItemView::new(toast)
                    .closing(is_closing)
                    .hovered_button(hov_btn)
                    .on_dismiss(move |window, cx| {
                        if let Some(ref h) = on_dismiss_cb {
                            h(t_id_str, window, cx);
                        }
                    })
                    .on_hover_button(move |idx, is_hov, window, cx| {
                        if let Some(ref h) = on_hov_cb {
                            h(t_id_str, idx, is_hov, window, cx);
                        }
                    });

                if depth == 0 {
                    overlay_box = overlay_box.child(item_el);
                } else {
                    let offset_y = match depth {
                        1 => -10.0,
                        _ => -20.0,
                    };
                    let scale_factor = match depth {
                        1 => 0.94,
                        _ => 0.88,
                    };
                    let opacity = match depth {
                        1 => 0.85,
                        _ => 0.70,
                    };

                    let layer = div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .w_full()
                        .mt(px(offset_y))
                        .opacity(opacity)
                        .child(item_el);

                    let _ = scale_factor;
                    overlay_box = overlay_box.child(layer);
                }
            }

            let _ = total_count;
            stack_container = stack_container.child(overlay_box);
        }

        let mut positioned = div().absolute();
        positioned = match self.position {
            ToastPosition::BottomRight => positioned.bottom(px(16.0)).right(px(16.0)),
            ToastPosition::BottomLeft => positioned.bottom(px(16.0)).left(px(16.0)),
            ToastPosition::TopRight => positioned.top(px(16.0)).right(px(16.0)),
            ToastPosition::TopLeft => positioned.top(px(16.0)).left(px(16.0)),
            ToastPosition::BottomCenter => positioned.bottom(px(16.0)).left_1_2().ml(px(-170.0)),
            ToastPosition::TopCenter => positioned.top(px(16.0)).left_1_2().ml(px(-170.0)),
        };

        positioned.child(stack_container).into_any_element()
    }
}

