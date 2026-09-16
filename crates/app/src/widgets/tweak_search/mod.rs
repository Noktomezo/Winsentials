use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window, deferred, div, ease_in_out, px,
};

use crate::entities::tweaks::{TweakSearchResult, search_tweaks};
use crate::shared::theme::Theme;
use crate::shared::ui::SearchInput;
use crate::shared::ui::icon::Icon;

pub mod result_card;
pub use result_card::TweakResultCard;

pub type TweakSearchSelectHandler =
    Arc<dyn Fn(TweakSearchResult, &mut Window, &mut App) + Send + Sync + 'static>;

const SEARCH_WIDTH: f32 = 280.0;

#[derive(Clone, Default)]
pub struct TweakSearchState {
    pub query: String,
    pub focused: bool,
    pub hovered: bool,
    pub selection: Option<(usize, usize)>,
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
    pub is_closing: bool,
    pub cached_results: Vec<TweakSearchResult>,
}

fn start_closing(state_entity: &gpui::Entity<TweakSearchState>, cx: &mut App) {
    state_entity.update(cx, |s, cx| {
        if s.is_closing {
            return;
        }
        s.is_closing = true;
        s.selected_index = None;
        s.hovered_index = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(140))
                .await;
            this.update(cx, |s, cx| {
                if s.is_closing {
                    s.is_closing = false;
                    s.cached_results.clear();
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    });
}

#[derive(IntoElement)]
pub struct TweakSearchWidget {
    windows_build: u32,
    on_select: Option<TweakSearchSelectHandler>,
}

impl TweakSearchWidget {
    #[must_use]
    pub fn new(windows_build: u32) -> Self {
        Self {
            windows_build,
            on_select: None,
        }
    }

    #[must_use]
    pub fn on_select(
        mut self,
        handler: impl Fn(TweakSearchResult, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }
}

fn build_empty_result_state(theme: &Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .py(px(24.0))
        .w_full()
        .child(
            Icon::new("icons/search.svg")
                .size(px(16.0))
                .color(theme.text_muted),
        )
        .child(
            div()
                .text_size(px(13.0))
                .text_color(theme.text_muted)
                .child(rust_i18n::t!("dashboard.no_tweaks_found").to_string()),
        )
        .into_any_element()
}

impl RenderOnce for TweakSearchWidget {
    #[allow(clippy::too_many_lines)]
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let state_entity =
            window.use_keyed_state("tweak_search_state", cx, |_, _| TweakSearchState::default());

        let state = state_entity.read(cx).clone();
        let query = state.query.clone();
        let focused = state.focused;
        let selected_index = state.selected_index;
        let hovered_index = state.hovered_index;

        let results = if !query.trim().is_empty() {
            let current = search_tweaks(&query, self.windows_build);
            state_entity.update(cx, |s, _| {
                s.cached_results.clone_from(&current);
            });
            current
        } else if state.is_closing {
            state.cached_results.clone()
        } else {
            Vec::new()
        };

        let is_open = focused && !query.trim().is_empty() && !state.is_closing;
        let show_dropdown = is_open || state.is_closing;

        let state_for_change = state_entity.clone();
        let on_change = move |new_val: String, _window: &mut Window, cx: &mut App| {
            let was_open = !state_for_change.read(cx).query.trim().is_empty();
            let will_be_empty = new_val.trim().is_empty();
            if was_open && will_be_empty {
                state_for_change.update(cx, |s, _| {
                    s.query = new_val;
                });
                start_closing(&state_for_change, cx);
            } else {
                state_for_change.update(cx, |s, cx| {
                    s.query = new_val;
                    s.is_closing = false;
                    s.selected_index = None;
                    cx.notify();
                });
            }
        };

        let state_for_hover = state_entity.clone();
        let on_hover = move |hovered: &bool, _window: &mut Window, cx: &mut App| {
            state_for_hover.update(cx, |s, cx| {
                s.hovered = *hovered;
                cx.notify();
            });
        };

        let state_for_focus = state_entity.clone();
        let on_focus_change = move |is_focused: bool, _window: &mut Window, cx: &mut App| {
            let was_open = {
                let s = state_for_focus.read(cx);
                s.focused && !s.query.trim().is_empty()
            };
            if was_open && !is_focused {
                state_for_focus.update(cx, |s, _| {
                    s.focused = false;
                });
                start_closing(&state_for_focus, cx);
            } else {
                state_for_focus.update(cx, |s, cx| {
                    s.focused = is_focused;
                    if !is_focused {
                        s.selected_index = None;
                    }
                    cx.notify();
                });
            }
        };

        let state_for_sel = state_entity.clone();
        let on_selection_change =
            move |sel: Option<(usize, usize)>, _window: &mut Window, cx: &mut App| {
                state_for_sel.update(cx, |s, cx| {
                    s.selection = sel;
                    cx.notify();
                });
            };

        let on_select_cb = self.on_select.clone();
        let state_for_submit = state_entity.clone();
        let results_for_submit = results.clone();
        let is_closing_submit = state.is_closing;
        let on_submit = move |_submitted: String, window: &mut Window, cx: &mut App| {
            if is_closing_submit {
                return;
            }
            if let Some(target) = selected_index
                .and_then(|idx| results_for_submit.get(idx))
                .or_else(|| results_for_submit.first())
            {
                let target_clone = target.clone();
                state_for_submit.update(cx, |s, cx| {
                    s.query.clear();
                    s.focused = false;
                    s.is_closing = false;
                    s.cached_results.clear();
                    s.selected_index = None;
                    cx.notify();
                });
                if let Some(ref h) = on_select_cb {
                    h(target_clone, window, cx);
                }
            }
        };

        let focus_handle = window.use_keyed_state("tweak_search_fh", cx, |_, cx| cx.focus_handle());
        let scroll_handle =
            window.use_keyed_state("tweak_search_sh", cx, |_, _| gpui::ScrollHandle::new());

        let search_input = SearchInput::new("dashboard_tweak_search_input", &query)
            .placeholder(rust_i18n::t!("dashboard.search_tweaks_placeholder").to_string())
            .width(px(SEARCH_WIDTH))
            .focused(focused)
            .hovered(state.hovered)
            .selection(state.selection)
            .track_focus(focus_handle.read(cx))
            .on_change(on_change)
            .on_hover(on_hover)
            .on_focus_change(on_focus_change)
            .on_selection_change(on_selection_change)
            .on_submit(on_submit);

        let dropdown_element = if show_dropdown {
            let mut list_col = div()
                .id(ElementId::Name("tweak_search_results_list".into()))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .w_full()
                .max_h(px(320.0))
                .overflow_y_hidden()
                .track_scroll(scroll_handle.read(cx));

            if results.is_empty() {
                list_col = list_col.child(build_empty_result_state(&theme));
            } else {
                for (idx, result) in results.into_iter().enumerate() {
                    let is_sel = selected_index == Some(idx);
                    let is_hov = hovered_index == Some(idx);
                    let state_for_item_select = state_entity.clone();
                    let on_select_handler = self.on_select.clone();
                    let is_closing = state.is_closing;

                    let state_for_item_hov = state_entity.clone();
                    let card = TweakResultCard::new(idx, result, is_sel, is_hov)
                        .on_select(move |res, window, cx| {
                            if is_closing {
                                return;
                            }
                            state_for_item_select.update(cx, |s, cx| {
                                s.query.clear();
                                s.focused = false;
                                s.is_closing = false;
                                s.cached_results.clear();
                                s.selected_index = None;
                                cx.notify();
                            });
                            if let Some(ref h) = on_select_handler {
                                h(res, window, cx);
                            }
                        })
                        .on_hover(move |idx, hov, _window, cx| {
                            state_for_item_hov.update(cx, |s, cx| {
                                s.hovered_index = if hov { Some(idx) } else { None };
                                cx.notify();
                            });
                        });

                    list_col = list_col.child(card);
                }
            }

            let mut box_el = div()
                .id(ElementId::Name("tweak_search_dropdown_box".into()))
                .absolute()
                .top(px(40.0))
                .left_0()
                .w_full()
                .p(px(6.0))
                .rounded(px(10.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .shadow_lg()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_click(|_, _, cx| cx.stop_propagation());

            let state_for_out = state_entity.clone();
            box_el = box_el.on_mouse_down_out(move |_, _window, cx| {
                start_closing(&state_for_out, cx);
            });

            let reduce_motion = cx.reduce_motion();
            let animated_box: AnyElement = if reduce_motion {
                box_el.child(list_col).into_any_element()
            } else if state.is_closing {
                box_el
                    .child(list_col)
                    .with_animation(
                        ElementId::Name("tweak_search_dropdown_close".into()),
                        Animation::new(Duration::from_millis(140)).with_easing(ease_in_out),
                        move |menu, delta| {
                            let offset_y = -delta * 4.0;
                            menu.opacity(1.0 - delta).mt(px(offset_y))
                        },
                    )
                    .into_any_element()
            } else {
                box_el
                    .child(list_col)
                    .with_animation(
                        ElementId::Name("tweak_search_dropdown_open".into()),
                        Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                        move |menu, delta| {
                            let offset_y = (1.0 - delta) * -6.0;
                            menu.opacity(delta).mt(px(offset_y))
                        },
                    )
                    .into_any_element()
            };

            Some(animated_box)
        } else {
            None
        };

        let state_for_keydown = state_entity;
        let mut container = div()
            .id(ElementId::Name("tweak_search_container".into()))
            .relative()
            .w(px(SEARCH_WIDTH))
            .on_key_down(move |event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "down" || key == "arrowdown" {
                    state_for_keydown.update(cx, |s, cx| {
                        s.selected_index = Some(s.selected_index.map_or(0, |i| i + 1));
                        cx.notify();
                    });
                } else if key == "up" || key == "arrowup" {
                    state_for_keydown.update(cx, |s, cx| {
                        s.selected_index =
                            Some(s.selected_index.map_or(0, |i| i.saturating_sub(1)));
                        cx.notify();
                    });
                } else if key == "escape" {
                    start_closing(&state_for_keydown, cx);
                    state_for_keydown.update(cx, |s, cx| {
                        s.query.clear();
                        s.focused = false;
                        s.selected_index = None;
                        cx.notify();
                    });
                }
            })
            .child(search_input);

        if let Some(dropdown) = dropdown_element {
            container = container.child(deferred(dropdown).with_priority(100));
        }

        container
    }
}
