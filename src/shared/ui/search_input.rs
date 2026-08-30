use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, DefiniteLength, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type SearchChangeHandler = Arc<dyn Fn(String, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct SearchInput {
    id: ElementId,
    id_str: String,
    value: String,
    placeholder: SharedString,
    width: DefiniteLength,
    focus_handle: Option<FocusHandle>,
    on_change: Option<SearchChangeHandler>,
}

impl SearchInput {
    #[must_use]
    pub fn new(id: impl Into<String>, value: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: ElementId::Name(id_str.clone().into()),
            id_str,
            value: value.into(),
            placeholder: rust_i18n::t!("startup.search_placeholder").into(),
            width: px(220.0).into(),
            focus_handle: None,
            on_change: None,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for SearchInput {
    #[allow(clippy::too_many_lines)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let is_focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|f| f.is_focused(window));

        let current_val = self.value.clone();
        let on_change_key = self.on_change.clone();
        let on_change_clear = self.on_change.clone();

        let focus_to_grab = self.focus_handle.clone();

        let mut input_box = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .gap(px(8.0))
            .h(px(32.0))
            .w(self.width)
            .px(px(10.0))
            .rounded_md()
            .bg(theme.input_bg)
            .border_1()
            .border_color(if is_focused {
                theme.accent_blue
            } else {
                theme.input_border
            })
            .cursor_text()
            .hover(move |s| {
                if is_focused {
                    s
                } else {
                    s.border_color(theme.card_border)
                }
            })
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(ref f) = focus_to_grab {
                    f.focus(window, cx);
                }
                cx.stop_propagation();
            });

        if let Some(ref f) = self.focus_handle {
            input_box = input_box.track_focus(f);
        }

        input_box = input_box.on_key_down(move |event: &KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            if key == "backspace" {
                let mut q = current_val.clone();
                if q.pop().is_some() {
                    if let Some(ref h) = on_change_key {
                        h(q, window, cx);
                    }
                }
            } else if key == "escape" {
                if let Some(ref h) = on_change_key {
                    h(String::new(), window, cx);
                }
            } else {
                let text_to_insert = event.keystroke.key_char.clone().or_else(|| {
                    if key.chars().count() == 1
                        && !event.keystroke.modifiers.control
                        && !event.keystroke.modifiers.alt
                        && !event.keystroke.modifiers.platform
                    {
                        Some(key.to_string())
                    } else {
                        None
                    }
                });

                if let Some(text) = text_to_insert {
                    let mut q = current_val.clone();
                    q.push_str(&text);
                    if let Some(ref h) = on_change_key {
                        h(q, window, cx);
                    }
                }
            }
        });

        input_box
            .child(
                Icon::new("icons/search.svg")
                    .size(px(14.0))
                    .color(if is_focused {
                        theme.accent_blue
                    } else {
                        theme.text_muted
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(if self.value.is_empty() {
                        theme.text_muted
                    } else {
                        theme.text_primary
                    })
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(if self.value.is_empty() {
                        self.placeholder.clone()
                    } else {
                        self.value.clone().into()
                    }),
            )
            .when(!self.value.is_empty(), |this| {
                this.child(
                    div()
                        .id(ElementId::Name(format!("{}_clear", self.id_str).into()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.0))
                        .rounded_full()
                        .hover(move |s| s.bg(theme.button_hover))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            if let Some(ref h) = on_change_clear {
                                h(String::new(), window, cx);
                            }
                            cx.stop_propagation();
                        })
                        .child(
                            Icon::new("icons/x.svg")
                                .size(px(12.0))
                                .color(theme.text_muted),
                        ),
                )
            })
    }
}
