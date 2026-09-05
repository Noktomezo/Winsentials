use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Rgba, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, div, ease_in_out, px,
};

use crate::components::icon::Icon;
use crate::theme::Theme;
#[allow(clippy::wildcard_imports)]
use super::types::*;

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

