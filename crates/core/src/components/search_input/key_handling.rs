use gpui::{App, KeyDownEvent, Window};

use super::types::{
    SearchChangeHandler, SearchFocusHandler, SearchSelectionHandler, SearchSubmitHandler,
};

#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
pub fn handle_key_down(
    event: &KeyDownEvent,
    current_val: &str,
    current_sel: Option<(usize, usize)>,
    on_change_key: Option<&SearchChangeHandler>,
    on_sel_cb: Option<&SearchSelectionHandler>,
    on_escape_cb: Option<&SearchFocusHandler>,
    on_submit_cb: Option<&SearchSubmitHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let key = event.keystroke.key.as_str();
    let is_ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

    if is_ctrl && (key == "a" || key == "A" || key == "ф" || key == "Ф") {
        let count = current_val.chars().count();
        if count > 0 {
            if let Some(h) = on_sel_cb {
                h(Some((0, count)), window, cx);
            }
        }
    } else if is_ctrl && (key == "c" || key == "C" || key == "с" || key == "С") {
        if let Some((s, e)) = current_sel {
            let count = current_val.chars().count();
            let start = s.min(count);
            let end = e.min(count).max(start);
            let sel_text: String = current_val.chars().skip(start).take(end - start).collect();
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
                if let Some(h) = on_sel_cb {
                    h(None, window, cx);
                }
                if let Some(h) = on_change_key {
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
                    let mut res = current_val.to_string();
                    res.push_str(&text);
                    res
                };
                q.retain(|c| c != '\r' && c != '\n');
                if let Some(h) = on_sel_cb {
                    h(None, window, cx);
                }
                if let Some(h) = on_change_key {
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
            if let Some(h) = on_sel_cb {
                h(None, window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        } else if key == "backspace" {
            let mut q = current_val.to_string();
            if q.pop().is_some() {
                if let Some(h) = on_change_key {
                    h(q, window, cx);
                }
            }
        }
    } else if key == "escape" {
        if current_sel.is_some() {
            if let Some(h) = on_sel_cb {
                h(None, window, cx);
            }
        } else {
            if let Some(h) = on_change_key {
                h(String::new(), window, cx);
            }
            if let Some(h) = on_escape_cb {
                h(false, window, cx);
            }
        }
    } else if key == "enter" {
        if let Some(h) = on_submit_cb {
            h(current_val.to_string(), window, cx);
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
                let mut res = current_val.to_string();
                res.push_str(&text);
                res
            };
            if let Some(h) = on_sel_cb {
                h(None, window, cx);
            }
            if let Some(h) = on_change_key {
                h(q, window, cx);
            }
        }
    }
}
