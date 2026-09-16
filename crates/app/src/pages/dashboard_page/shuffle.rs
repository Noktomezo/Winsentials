use std::collections::HashMap;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, SpringAnimation,
    SpringConfig, Styled, Window, div, px,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridShuffleState {
    pub slots: HashMap<String, usize>,
}

#[must_use]
pub fn calculate_shuffle_delta(
    prev_index: usize,
    curr_index: usize,
    col_stride: f32,
    row_stride: f32,
) -> (f32, f32) {
    let prev_col = (prev_index % 2) as f32;
    let prev_row = (prev_index / 2) as f32;
    let curr_col = (curr_index % 2) as f32;
    let curr_row = (curr_index / 2) as f32;

    let delta_x = (prev_col - curr_col) * col_stride;
    let delta_y = (prev_row - curr_row) * row_stride;
    (delta_x, delta_y)
}

#[allow(clippy::too_many_arguments)]
pub fn wrap_shuffled_card(
    card_id: &'static str,
    card_element: AnyElement,
    current_index: usize,
    prev_slots: &HashMap<String, usize>,
    col_stride: Pixels,
    row_stride: Pixels,
    reduce_motion: bool,
) -> AnyElement {
    if reduce_motion {
        return card_element;
    }

    let prev_index = prev_slots.get(card_id).copied();

    match prev_index {
        Some(prev_idx) if prev_idx != current_index => {
            let (dx, dy) = calculate_shuffle_delta(
                prev_idx,
                current_index,
                col_stride / px(1.0),
                row_stride / px(1.0),
            );

            // Snappy natural spring for card repositioning
            let spring = SpringAnimation::new(SpringConfig::new(280.0, 26.0, 1.0))
                .to(1.0)
                .with_epsilon(0.005);

            div()
                .w_full()
                .child(card_element)
                .with_spring(
                    ElementId::Name(
                        format!("shuffle_pos_{card_id}_{prev_idx}_to_{current_index}").into(),
                    ),
                    spring,
                    move |container, val| {
                        let progress = val.clamp(0.0, 1.0);
                        let remaining = 1.0 - progress;
                        let offset_x = dx * remaining;
                        let offset_y = dy * remaining;
                        container.relative().left(px(offset_x)).top(px(offset_y))
                    },
                )
                .into_any_element()
        }
        None if !prev_slots.is_empty() => {
            // Newly inserted device card: fade in smoothly
            let spring = SpringAnimation::new(SpringConfig::new(280.0, 26.0, 1.0))
                .to(1.0)
                .with_epsilon(0.005);

            div()
                .w_full()
                .child(card_element)
                .with_spring(
                    ElementId::Name(format!("shuffle_enter_{card_id}").into()),
                    spring,
                    move |container, val| {
                        let progress = val.clamp(0.0, 1.0);
                        container.opacity(progress)
                    },
                )
                .into_any_element()
        }
        _ => card_element,
    }
}

pub fn update_shuffle_tracker(
    window: &mut Window,
    cx: &mut App,
    current_slots: HashMap<String, usize>,
) {
    let tracker = window.use_keyed_state("dashboard_grid_shuffle", cx, |_, _| {
        GridShuffleState::default()
    });
    let needs_update = tracker.read(cx).slots != current_slots;

    if needs_update {
        window.on_next_frame(move |_window, cx| {
            tracker.update(cx, |state, _cx| {
                state.slots = current_slots;
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_shuffle_delta_same_slot() {
        let (dx, dy) = calculate_shuffle_delta(2, 2, 400.0, 76.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn test_calculate_shuffle_delta_horizontal_shift() {
        // Shift from col 1 (index 1) to col 0 (index 0) in row 0
        let (dx, dy) = calculate_shuffle_delta(1, 0, 400.0, 76.0);
        assert_eq!(dx, 400.0);
        assert_eq!(dy, 0.0);

        // Shift from col 0 (index 0) to col 1 (index 1) in row 0
        let (dx, dy) = calculate_shuffle_delta(0, 1, 400.0, 76.0);
        assert_eq!(dx, -400.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn test_calculate_shuffle_delta_vertical_shift() {
        // Shift from row 0 (index 0) to row 1 (index 2), col 0 -> col 0
        let (dx, dy) = calculate_shuffle_delta(0, 2, 400.0, 76.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, -76.0);

        // Shift from row 1 (index 3) to row 0 (index 1), col 1 -> col 1
        let (dx, dy) = calculate_shuffle_delta(3, 1, 400.0, 76.0);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 76.0);
    }

    #[test]
    fn test_calculate_shuffle_delta_diagonal_shift() {
        // When a card is inserted at index 1:
        // Card originally at index 1 (col 1, row 0) shifts to index 2 (col 0, row 1)
        let (dx, dy) = calculate_shuffle_delta(1, 2, 400.0, 76.0);
        assert_eq!(dx, 400.0);
        assert_eq!(dy, -76.0);
    }
}
