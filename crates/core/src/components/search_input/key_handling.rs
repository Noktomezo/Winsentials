use gpui::{App, KeyDownEvent, Window};

use super::types::{
    SearchChangeHandler, SearchFocusHandler, SearchSelectionHandler, SearchSubmitHandler,
};

fn find_prev_word_boundary(chars: &[char], from: usize) -> usize {
    if from == 0 {
        return 0;
    }
    let mut i = from;
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    if i > 0 && chars[i - 1].is_alphanumeric() {
        while i > 0 && chars[i - 1].is_alphanumeric() {
            i -= 1;
        }
    } else if i > 0 {
        while i > 0 && !chars[i - 1].is_alphanumeric() && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
    }
    i
}

fn find_next_word_boundary(chars: &[char], from: usize) -> usize {
    let len = chars.len();
    if from >= len {
        return len;
    }
    let mut i = from;
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    if i < len && chars[i].is_alphanumeric() {
        while i < len && chars[i].is_alphanumeric() {
            i += 1;
        }
    } else if i < len {
        while i < len && !chars[i].is_alphanumeric() && !chars[i].is_whitespace() {
            i += 1;
        }
    }
    i
}

#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
pub fn handle_key_down(
    event: &KeyDownEvent,
    current_val: &str,
    current_sel: Option<(usize, usize)>,
    max_length: Option<usize>,
    on_change_key: Option<&SearchChangeHandler>,
    on_sel_cb: Option<&SearchSelectionHandler>,
    on_escape_cb: Option<&SearchFocusHandler>,
    on_submit_cb: Option<&SearchSubmitHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let key = event.keystroke.key.as_str();
    let is_ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
    let is_shift = event.keystroke.modifiers.shift;
    let is_alt = event.keystroke.modifiers.alt;

    let chars: Vec<char> = current_val.chars().collect();
    let char_count = chars.len();
    let (anchor, head) = current_sel.unwrap_or((char_count, char_count));
    let anchor = anchor.min(char_count);
    let head = head.min(char_count);
    let has_selection = anchor != head;
    let sel_start = anchor.min(head);
    let sel_end = anchor.max(head);

    if is_ctrl && !is_shift && !is_alt && (key == "a" || key == "A" || key == "ф" || key == "Ф") {
        if char_count > 0 {
            if let Some(h) = on_sel_cb {
                h(Some((0, char_count)), window, cx);
            }
        }
    } else if is_ctrl
        && !is_shift
        && !is_alt
        && (key == "c" || key == "C" || key == "с" || key == "С")
    {
        if has_selection {
            let sel_text: String = chars[sel_start..sel_end].iter().collect();
            if !sel_text.is_empty() {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(sel_text));
            }
        }
    } else if is_ctrl
        && !is_shift
        && !is_alt
        && (key == "x" || key == "X" || key == "ч" || key == "Ч")
    {
        if has_selection {
            let sel_text: String = chars[sel_start..sel_end].iter().collect();
            if !sel_text.is_empty() {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(sel_text));
            }
            let mut res = String::new();
            res.extend(&chars[..sel_start]);
            res.extend(&chars[sel_end..]);
            if let Some(h) = on_sel_cb {
                h(Some((sel_start, sel_start)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        }
    } else if is_ctrl
        && !is_shift
        && !is_alt
        && (key == "v" || key == "V" || key == "м" || key == "М")
    {
        if let Some(clip) = cx.read_from_clipboard() {
            if let Some(text) = clip.text() {
                let mut clean_text = text;
                clean_text.retain(|c| c != '\r' && c != '\n');
                let clean_count = clean_text.chars().count();
                let available = if let Some(max) = max_length {
                    let cur_without_sel = char_count - (sel_end - sel_start);
                    max.saturating_sub(cur_without_sel)
                } else {
                    clean_count
                };

                if available > 0 {
                    let paste_str: String = clean_text.chars().take(available).collect();
                    let paste_len = paste_str.chars().count();
                    let mut res = String::new();
                    res.extend(&chars[..sel_start]);
                    res.push_str(&paste_str);
                    res.extend(&chars[sel_end..]);
                    let new_cursor = sel_start + paste_len;
                    if let Some(h) = on_sel_cb {
                        h(Some((new_cursor, new_cursor)), window, cx);
                    }
                    if let Some(h) = on_change_key {
                        h(res, window, cx);
                    }
                }
            }
        }
    } else if key == "left" || key == "arrowleft" {
        let new_head = if is_ctrl {
            find_prev_word_boundary(&chars, head)
        } else if is_shift {
            head.saturating_sub(1)
        } else if has_selection {
            sel_start
        } else {
            head.saturating_sub(1)
        };

        let new_sel = if is_shift {
            Some((anchor, new_head))
        } else {
            Some((new_head, new_head))
        };
        if let Some(h) = on_sel_cb {
            h(new_sel, window, cx);
        }
    } else if key == "right" || key == "arrowright" {
        let new_head = if is_ctrl {
            find_next_word_boundary(&chars, head)
        } else if is_shift {
            (head + 1).min(char_count)
        } else if has_selection {
            sel_end
        } else {
            (head + 1).min(char_count)
        };

        let new_sel = if is_shift {
            Some((anchor, new_head))
        } else {
            Some((new_head, new_head))
        };
        if let Some(h) = on_sel_cb {
            h(new_sel, window, cx);
        }
    } else if key == "home" {
        let new_sel = if is_shift {
            Some((anchor, 0))
        } else {
            Some((0, 0))
        };
        if let Some(h) = on_sel_cb {
            h(new_sel, window, cx);
        }
    } else if key == "end" {
        let new_sel = if is_shift {
            Some((anchor, char_count))
        } else {
            Some((char_count, char_count))
        };
        if let Some(h) = on_sel_cb {
            h(new_sel, window, cx);
        }
    } else if key == "backspace" {
        if has_selection {
            let mut res = String::new();
            res.extend(&chars[..sel_start]);
            res.extend(&chars[sel_end..]);
            if let Some(h) = on_sel_cb {
                h(Some((sel_start, sel_start)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        } else if head > 0 {
            let del_idx = if is_ctrl {
                find_prev_word_boundary(&chars, head)
            } else {
                head - 1
            };
            let mut res = String::new();
            res.extend(&chars[..del_idx]);
            res.extend(&chars[head..]);
            if let Some(h) = on_sel_cb {
                h(Some((del_idx, del_idx)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        }
    } else if key == "delete" {
        if has_selection {
            let mut res = String::new();
            res.extend(&chars[..sel_start]);
            res.extend(&chars[sel_end..]);
            if let Some(h) = on_sel_cb {
                h(Some((sel_start, sel_start)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        } else if head < char_count {
            let del_end = if is_ctrl {
                find_next_word_boundary(&chars, head)
            } else {
                head + 1
            };
            let mut res = String::new();
            res.extend(&chars[..head]);
            res.extend(&chars[del_end..]);
            if let Some(h) = on_sel_cb {
                h(Some((head, head)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        }
    } else if key == "escape" {
        if has_selection {
            if let Some(h) = on_sel_cb {
                h(Some((head, head)), window, cx);
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
            if key.chars().count() == 1 && !is_ctrl && !is_alt {
                Some(key.to_string())
            } else {
                None
            }
        });

        if let Some(text) = text_to_insert {
            let insert_len = text.chars().count();
            let new_char_count = char_count - (sel_end - sel_start) + insert_len;
            if let Some(max) = max_length {
                if new_char_count > max {
                    return;
                }
            }

            let mut res = String::new();
            res.extend(&chars[..sel_start]);
            res.push_str(&text);
            res.extend(&chars[sel_end..]);
            let new_cursor = sel_start + insert_len;
            if let Some(h) = on_sel_cb {
                h(Some((new_cursor, new_cursor)), window, cx);
            }
            if let Some(h) = on_change_key {
                h(res, window, cx);
            }
        }
    }
}
