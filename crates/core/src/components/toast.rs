use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, div, ease_in_out, px,
};

use crate::components::icon::Icon;
use crate::theme::Theme;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
    BottomCenter,
    TopCenter,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Destructive,
}

pub type ToastActionHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type ToastDismissHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type ToastButtonHoverHandler = Arc<dyn Fn(usize, &bool, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct ToastButton {
    pub label: SharedString,
    pub variant: ToastButtonVariant,
    pub icon: Option<SharedString>,
    pub on_click: Option<ToastActionHandler>,
    pub full_width: bool,
}

#[allow(dead_code)]
impl ToastButton {
    #[must_use]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ToastButtonVariant::Primary,
            icon: None,
            on_click: None,
            full_width: false,
        }
    }

    #[must_use]
    pub const fn variant(mut self, variant: ToastButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub const fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastProgress {
    pub value: f32, // 0.0 .. 1.0
    pub label: Option<SharedString>,
}

#[derive(Clone)]
pub struct ToastData {
    pub id: SharedString,
    pub variant: ToastVariant,
    pub position: ToastPosition,
    pub icon: Option<SharedString>,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub buttons: Vec<ToastButton>,
    pub progress: Option<ToastProgress>,
    pub duration: Option<Duration>,
    pub count: usize,
}

#[allow(dead_code)]
impl ToastData {
    #[must_use]
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: ToastVariant::Default,
            position: ToastPosition::BottomRight,
            icon: None,
            title: title.into(),
            description: None,
            buttons: Vec::new(),
            progress: None,
            duration: Some(Duration::from_secs(5)),
            count: 1,
        }
    }

    #[must_use]
    pub const fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub const fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn button(mut self, button: ToastButton) -> Self {
        self.buttons.push(button);
        self
    }

    #[must_use]
    pub fn buttons(mut self, buttons: Vec<ToastButton>) -> Self {
        self.buttons = buttons;
        self
    }

    #[must_use]
    pub fn progress(mut self, progress: Option<ToastProgress>) -> Self {
        self.progress = progress;
        self
    }

    #[must_use]
    pub const fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub const fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

#[derive(IntoElement)]
pub struct ToastItemView {
    data: ToastData,
    closing: bool,
    hovered_button: Option<usize>,
    on_dismiss: Option<ToastDismissHandler>,
    on_hover_button: Option<ToastButtonHoverHandler>,
}

impl ToastItemView {
    #[must_use]
    pub fn new(data: ToastData) -> Self {
        Self {
            data,
            closing: false,
            hovered_button: None,
            on_dismiss: None,
            on_hover_button: None,
        }
    }

    #[must_use]
    pub const fn closing(mut self, closing: bool) -> Self {
        self.closing = closing;
        self
    }

    #[must_use]
    pub const fn hovered_button(mut self, index: Option<usize>) -> Self {
        self.hovered_button = index;
        self
    }

    #[must_use]
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_button(
        mut self,
        handler: impl Fn(usize, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_button = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ToastItemView {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let id_str = self.data.id.clone();
        let is_closing = self.closing;
        let on_dismiss = self.on_dismiss;
        let on_hover_button = self.on_hover_button;

        // Accent / Icon color according to variant
        let accent_color: Rgba = match self.data.variant {
            ToastVariant::Success => theme.accent_green,
            ToastVariant::Warning => theme.accent_orange,
            ToastVariant::Error => theme.accent_red,
            ToastVariant::Default | ToastVariant::Info => theme.accent_blue,
        };

        // Icon resolution
        let icon_path = self.data.icon.unwrap_or_else(|| match self.data.variant {
            ToastVariant::Default => "icons/bell.svg".into(),
            ToastVariant::Success => "icons/circle-check.svg".into(),
            ToastVariant::Warning => "icons/triangle-alert.svg".into(),
            ToastVariant::Error => "icons/circle-alert.svg".into(),
            ToastVariant::Info => "icons/info.svg".into(),
        });

        // Top Header matching TweakCard style
        let icon_box = div()
            .size(px(32.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .border_color(theme.input_border)
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .child(
                Icon::new(icon_path.to_string())
                    .size(px(16.0))
                    .color(accent_color),
            );

        let mut title_row = div().flex().items_center().gap(px(6.0)).h(px(16.0)).child(
            div()
                .text_size(px(13.5))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .child(self.data.title),
        );

        if self.data.count >= 2 {
            title_row = title_row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(px(4.5))
                    .h(px(14.0))
                    .rounded(px(3.0))
                    .bg(theme.input_bg)
                    .border_1()
                    .border_color(theme.input_border)
                    .text_size(px(10.0))
                    .line_height(px(10.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_muted)
                    .child(format!("{}", self.data.count)),
            );
        }

        let mut text_stack = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .flex_1()
            .min_w(px(0.0))
            .child(title_row);

        if let Some(desc) = self.data.description {
            text_stack = text_stack.child(
                div()
                    .text_size(px(11.5))
                    .line_height(px(14.0))
                    .font_weight(gpui::FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(desc),
            );
        }

        let dismiss_btn = on_dismiss.clone().map(|dismiss_cb| {
            let dismiss_btn_id = format!("toast_dismiss_{id_str}");
            div()
                .id(ElementId::Name(dismiss_btn_id.into()))
                .size(px(20.0))
                .rounded(px(4.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(theme.text_muted)
                .hover(|s| s.text_color(theme.text_primary).bg(theme.input_bg))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    dismiss_cb(window, cx);
                })
                .child(Icon::new("icons/x.svg").size(px(14.0)))
        });

        let header_row = div()
            .flex()
            .items_start()
            .justify_between()
            .w_full()
            .gap(px(10.0))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(12.0))
                    .flex_1()
                    .min_w(px(0.0))
                    .child(icon_box)
                    .child(text_stack),
            )
            .children(dismiss_btn);

        // Progress bar slot (if provided)
        let progress_el = if let Some(prog) = self.data.progress {
            let pct = prog.value.clamp(0.0, 1.0);
            let mut bar_container = div().flex().flex_col().gap(px(4.0)).w_full().mt(px(4.0));

            if let Some(lbl) = prog.label {
                bar_container = bar_container.child(
                    div()
                        .flex()
                        .justify_between()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(lbl)
                        .child(format!("{:.0}%", pct * 100.0)),
                );
            }

            let track = div()
                .w_full()
                .h(px(4.0))
                .rounded(px(2.0))
                .bg(theme.input_bg)
                .border_1()
                .border_color(theme.input_border)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .rounded(px(2.0))
                        .bg(accent_color)
                        .w(gpui::relative(pct)),
                );

            bar_container.child(track).into_any_element()
        } else {
            div().size(px(0.0)).into_any_element()
        };

        let has_buttons = !self.data.buttons.is_empty();

        let buttons_el = if has_buttons {
            let mut btns_row = div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .w_full()
                .mt(px(6.0));

            for (idx, btn) in self.data.buttons.into_iter().enumerate() {
                let is_hovered = self.hovered_button == Some(idx);
                let btn_target = if is_hovered { 1.0 } else { 0.0 };
                let btn_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
                    .to(btn_target)
                    .with_epsilon(0.005);

                let on_hover_btn_cb = on_hover_button.clone();
                let click_cb = btn.on_click;
                let dismiss_cb = on_dismiss.clone();
                let btn_id = format!("toast_btn_{idx}_{id_str}");

                let (bg_rest, border_rest, text_rest, bg_hover, border_hover, text_hover) =
                    match btn.variant {
                        ToastButtonVariant::Primary => (
                            theme.accent_blue,
                            theme.accent_blue,
                            Rgba {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            },
                            theme.accent_hover_bg,
                            theme.accent_blue,
                            theme.accent_blue,
                        ),
                        ToastButtonVariant::Secondary => (
                            theme.input_bg,
                            theme.input_border,
                            theme.text_primary,
                            theme.accent_hover_bg,
                            theme.accent_blue,
                            theme.accent_blue,
                        ),
                        ToastButtonVariant::Outline => (
                            Rgba {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            },
                            theme.input_border,
                            theme.text_primary,
                            theme.accent_hover_bg,
                            theme.accent_blue,
                            theme.accent_blue,
                        ),
                        ToastButtonVariant::Destructive => (
                            theme.accent_red,
                            theme.accent_red,
                            Rgba {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            },
                            theme.accent_red,
                            theme.accent_red,
                            Rgba {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            },
                        ),
                    };

                let btn_icon = btn
                    .icon
                    .map(|ic| Icon::new(ic.to_string()).size(px(13.0)).color(text_rest));

                let mut button_el = div()
                    .id(ElementId::Name(btn_id.clone().into()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(6.0));

                if btn.full_width {
                    button_el = button_el.w_full();
                } else {
                    button_el = button_el.flex_1();
                }

                let button_el = button_el
                    .h(px(30.0))
                    .px(px(12.0))
                    .rounded(px(5.0))
                    .border_1()
                    .cursor_pointer()
                    .text_size(px(12.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .on_hover(move |&hov, window, cx| {
                        if let Some(ref h) = on_hover_btn_cb {
                            h(idx, &hov, window, cx);
                        }
                    })
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_mouse_down(MouseButton::Right, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        if let Some(ref h) = click_cb {
                            h(window, cx);
                        }
                        if let Some(ref d) = dismiss_cb {
                            d(window, cx);
                        }
                    })
                    .with_spring(
                        ElementId::Name(format!("{btn_id}_spring").into()),
                        btn_spring,
                        move |el, val| {
                            let v = val.clamp(0.0, 1.0);
                            let bg = crate::motion::lerp_rgba(bg_rest, bg_hover, v);
                            let border = crate::motion::lerp_rgba(border_rest, border_hover, v);
                            let text = crate::motion::lerp_rgba(text_rest, text_hover, v);
                            el.bg(bg).border_color(border).text_color(text)
                        },
                    )
                    .children(btn_icon)
                    .child(btn.label);

                btns_row = btns_row.child(button_el);
            }

            btns_row.into_any_element()
        } else {
            div().size(px(0.0)).into_any_element()
        };

        let on_dismiss_card = on_dismiss;

        let card_body = div()
            .id(ElementId::Name(format!("toast_card_{id_str}").into()))
            .flex()
            .flex_col()
            .w(px(340.0))
            .p(px(14.0))
            .gap(px(10.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(theme.card_border)
            .bg(theme.card_bg)
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
            .on_mouse_up(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                if !has_buttons {
                    if let Some(ref d) = on_dismiss_card {
                        d(window, cx);
                    }
                }
            })
            .child(header_row)
            .child(progress_el)
            .child(buttons_el);

        let anim_name = if is_closing {
            format!("toast_exit_{id_str}")
        } else {
            format!("toast_enter_{id_str}")
        };

        if is_closing {
            card_body
                .with_animation(
                    ElementId::Name(anim_name.into()),
                    Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                    move |el, delta| {
                        let offset_y = delta * 12.0;
                        let opacity = 1.0 - delta;
                        el.opacity(opacity).mt(px(offset_y))
                    },
                )
                .into_any_element()
        } else {
            card_body
                .with_animation(
                    ElementId::Name(anim_name.into()),
                    Animation::new(Duration::from_millis(200)).with_easing(ease_in_out),
                    move |el, delta| {
                        let offset_y = (1.0 - delta) * 16.0;
                        let opacity = delta;
                        el.opacity(opacity).mt(px(offset_y))
                    },
                )
                .into_any_element()
        }
    }
}

pub type ToastDismissIdHandler = Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;
pub type ToastHoverBtnIdHandler = Arc<dyn Fn(&str, usize, &bool, &mut Window, &mut App) + 'static>;
pub type ToastStackHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

/// Sonner-style Stack component for multiple Toast notifications
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_data_builder() {
        let toast = ToastData::new("test_1", "Test Toast")
            .description("Description")
            .variant(ToastVariant::Success)
            .button(ToastButton::new("Action"));

        assert_eq!(toast.id, "test_1");
        assert_eq!(toast.title, "Test Toast");
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.buttons.len(), 1);
        assert_eq!(toast.count, 1);
    }
}
