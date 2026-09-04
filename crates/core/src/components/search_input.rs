use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, App, DefiniteLength, ElementId, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Window, div,
    ease_in_out, px,
};

use crate::components::icon::Icon;
use crate::theme::Theme;

pub type SearchChangeHandler = Arc<dyn Fn(String, &mut Window, &mut App) + 'static>;
pub type SearchHoverHandler = Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SearchFocusHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
pub type SearchSelectionHandler =
    Arc<dyn Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct SearchInput {
    id: ElementId,
    id_str: String,
    value: String,
    placeholder: SharedString,
    width: DefiniteLength,
    focused: bool,
    hovered: bool,
    selection: Option<(usize, usize)>,
    focus_handle: Option<FocusHandle>,
    on_change: Option<SearchChangeHandler>,
    on_hover: Option<SearchHoverHandler>,
    on_focus_change: Option<SearchFocusHandler>,
    on_selection_change: Option<SearchSelectionHandler>,
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
            focused: false,
            hovered: false,
            selection: None,
            focus_handle: None,
            on_change: None,
            on_hover: None,
            on_focus_change: None,
            on_selection_change: None,
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
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[must_use]
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[must_use]
    pub fn selection(mut self, selection: Option<(usize, usize)>) -> Self {
        self.selection = selection;
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

    #[must_use]
    pub fn on_hover(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_focus_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_focus_change = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_selection_change(
        mut self,
        handler: impl Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for SearchInput {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let is_focused = self.focused;
        let is_hovered = self.hovered;
        let current_sel = self.selection;

        let current_val = self.value.clone();
        let current_val_mouse = self.value.clone();
        let on_change_key = self.on_change.clone();
        let on_change_clear = self.on_change;
        let on_hover_cb = self.on_hover;
        let on_focus_cb = self.on_focus_change.clone();
        let on_focus_out_cb = self.on_focus_change.clone();
        let on_escape_cb = self.on_focus_change;
        let on_sel_cb = self.on_selection_change.clone();
        let on_sel_mouse_cb = self.on_selection_change.clone();
        let on_sel_out_cb = self.on_selection_change.clone();
        let on_sel_clear = self.on_selection_change;

        let focus_to_grab = self.focus_handle.clone();
        let id_str = self.id_str.clone();

        // Spring-driven border transition: 1.0 on focused (100%), 0.5 on hovered (50%), 0.0 on idle (0%)
        let trigger_target: f32 = if is_focused {
            1.0
        } else if is_hovered {
            0.5
        } else {
            0.0
        };

        let trigger_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(trigger_target)
            .with_epsilon(0.005);

        let neutral_border = theme.input_border;
        let blue_border = theme.accent_blue;
        let hover_blue_border = theme.accent_hover_bg;

        // Smoothly animated blinking caret
        let caret_anim_id = format!("{id_str}_caret_blink");
        let caret_el = div()
            .id(ElementId::Name(format!("{id_str}_caret").into()))
            .w(px(1.5))
            .h(px(14.0))
            .bg(theme.accent_blue)
            .rounded(px(1.0))
            .with_animation(
                ElementId::Name(caret_anim_id.into()),
                Animation::new(Duration::from_millis(850))
                    .repeat()
                    .with_easing(ease_in_out),
                move |el, delta| {
                    let wave = (delta * std::f32::consts::PI * 2.0).cos();
                    let alpha = (0.5 + 0.5 * wave).clamp(0.0, 1.0);
                    el.opacity(alpha)
                },
            );

        let mut input_box = div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(px(8.0))
            .h(px(32.0))
            .w(self.width)
            .px(px(10.0))
            .rounded(px(6.0))
            .bg(theme.input_bg)
            .border_1()
            .cursor_text()
            .on_hover(move |&hov, window, cx| {
                if let Some(ref h) = on_hover_cb {
                    h(&hov, window, cx);
                }
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                if let Some(ref f) = focus_to_grab {
                    f.focus(window, cx);
                }
                if let Some(ref h) = on_focus_cb {
                    h(true, window, cx);
                }
                if event.click_count >= 2 {
                    let count = current_val_mouse.chars().count();
                    if count > 0 {
                        if let Some(ref h) = on_sel_mouse_cb {
                            h(Some((0, count)), window, cx);
                        }
                    }
                } else if let Some(ref h) = on_sel_mouse_cb {
                    h(None, window, cx);
                }
                cx.stop_propagation();
            })
            .on_mouse_down_out(move |_, window, cx| {
                if let Some(ref h) = on_sel_out_cb {
                    h(None, window, cx);
                }
                if let Some(ref h) = on_focus_out_cb {
                    h(false, window, cx);
                }
            });

        if let Some(ref f) = self.focus_handle {
            input_box = input_box.track_focus(f);
        }

        input_box = input_box.on_key_down(move |event: &KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let is_ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

            if is_ctrl && (key == "a" || key == "A" || key == "ф" || key == "Ф") {
                let count = current_val.chars().count();
                if count > 0 {
                    if let Some(ref h) = on_sel_cb {
                        h(Some((0, count)), window, cx);
                    }
                }
            } else if is_ctrl && (key == "c" || key == "C" || key == "с" || key == "С") {
                if let Some((s, e)) = current_sel {
                    let count = current_val.chars().count();
                    let start = s.min(count);
                    let end = e.min(count).max(start);
                    let sel_text: String =
                        current_val.chars().skip(start).take(end - start).collect();
                    if !sel_text.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(sel_text));
                    }
                }
            } else if is_ctrl && (key == "x" || key == "X" || key == "ч" || key == "Ч") {
                if let Some((s, e)) = current_sel {
                    let chars: Vec<char> = current_val.chars().collect();
                    let start = s.min(chars.len());
                    let end = e.min(chars.len()).max(start);
                    let sel_text: String = chars[start..end].iter().collect();
                    if !sel_text.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(sel_text));
                        let mut res = String::new();
                        res.extend(&chars[..start]);
                        res.extend(&chars[end..]);
                        if let Some(ref h) = on_sel_cb {
                            h(None, window, cx);
                        }
                        if let Some(ref h) = on_change_key {
                            h(res, window, cx);
                        }
                    }
                }
            } else if is_ctrl && (key == "v" || key == "V" || key == "м" || key == "М") {
                if let Some(clip) = cx.read_from_clipboard() {
                    if let Some(text) = clip.text() {
                        let mut q = if let Some((s, e)) = current_sel {
                            let chars: Vec<char> = current_val.chars().collect();
                            let start = s.min(chars.len());
                            let end = e.min(chars.len()).max(start);
                            let mut res = String::new();
                            res.extend(&chars[..start]);
                            res.push_str(&text);
                            res.extend(&chars[end..]);
                            res
                        } else {
                            let mut res = current_val.clone();
                            res.push_str(&text);
                            res
                        };
                        q.retain(|c| c != '\r' && c != '\n');
                        if let Some(ref h) = on_sel_cb {
                            h(None, window, cx);
                        }
                        if let Some(ref h) = on_change_key {
                            h(q, window, cx);
                        }
                    }
                }
            } else if key == "backspace" || key == "delete" {
                if let Some((s, e)) = current_sel {
                    let chars: Vec<char> = current_val.chars().collect();
                    let start = s.min(chars.len());
                    let end = e.min(chars.len()).max(start);
                    let mut res = String::new();
                    res.extend(&chars[..start]);
                    res.extend(&chars[end..]);
                    if let Some(ref h) = on_sel_cb {
                        h(None, window, cx);
                    }
                    if let Some(ref h) = on_change_key {
                        h(res, window, cx);
                    }
                } else if key == "backspace" {
                    let mut q = current_val.clone();
                    if q.pop().is_some() {
                        if let Some(ref h) = on_change_key {
                            h(q, window, cx);
                        }
                    }
                }
            } else if key == "escape" {
                if current_sel.is_some() {
                    if let Some(ref h) = on_sel_cb {
                        h(None, window, cx);
                    }
                } else {
                    if let Some(ref h) = on_change_key {
                        h(String::new(), window, cx);
                    }
                    if let Some(ref h) = on_escape_cb {
                        h(false, window, cx);
                    }
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
                    let q = if let Some((s, e)) = current_sel {
                        let chars: Vec<char> = current_val.chars().collect();
                        let start = s.min(chars.len());
                        let end = e.min(chars.len()).max(start);
                        let mut res = String::new();
                        res.extend(&chars[..start]);
                        res.push_str(&text);
                        res.extend(&chars[end..]);
                        res
                    } else {
                        let mut res = current_val.clone();
                        res.push_str(&text);
                        res
                    };
                    if let Some(ref h) = on_sel_cb {
                        h(None, window, cx);
                    }
                    if let Some(ref h) = on_change_key {
                        h(q, window, cx);
                    }
                }
            }
        });

        let icon_color = if is_focused {
            theme.accent_blue
        } else if is_hovered {
            crate::motion::lerp_rgba(theme.text_muted, theme.accent_blue, 0.5)
        } else {
            theme.text_muted
        };

        let content_el = if self.value.is_empty() {
            div()
                .relative()
                .flex()
                .items_center()
                .when(is_focused, |this| {
                    this.child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(0.0))
                            .bottom(px(0.0))
                            .flex()
                            .items_center()
                            .child(caret_el),
                    )
                })
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_muted)
                        .truncate()
                        .child(self.placeholder.clone()),
                )
                .into_any_element()
        } else if let Some((start, end)) = self.selection {
            let char_count = self.value.chars().count();
            let s = start.min(char_count);
            let e = end.min(char_count).max(s);
            let chars: Vec<char> = self.value.chars().collect();
            let before: String = chars[..s].iter().collect();
            let sel: String = chars[s..e].iter().collect();
            let after: String = chars[e..].iter().collect();

            div()
                .flex()
                .items_center()
                .when(!before.is_empty(), |this| {
                    this.child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_primary)
                            .child(before),
                    )
                })
                .child(
                    div()
                        .px(px(1.0))
                        .rounded(px(2.0))
                        .bg(theme.accent_blue.opacity(0.35))
                        .text_xs()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_primary)
                        .child(sel),
                )
                .when(!after.is_empty(), |this| {
                    this.child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_primary)
                            .child(after),
                    )
                })
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_primary)
                        .child(self.value.clone()),
                )
                .when(is_focused, |this| this.child(caret_el))
                .into_any_element()
        };

        input_box
            .with_spring(
                ElementId::Name(format!("{id_str}_spring").into()),
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
            .child(
                Icon::new("icons/search.svg")
                    .size(px(14.0))
                    .color(icon_color),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(content_el),
            )
            .when(!self.value.is_empty(), |this| {
                this.child(
                    div()
                        .id(ElementId::Name(format!("{id_str}_clear").into()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.0))
                        .rounded_full()
                        .hover(move |s| s.bg(theme.button_hover))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            if let Some(ref h) = on_sel_clear {
                                h(None, window, cx);
                            }
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
