use gpui::{
    AnimationExt, AnyElement, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels,
    Point, SpringAnimation, SpringConfig, Styled, div, point, px,
};

pub const GRID_LAYOUT_SPRING: SpringConfig = SpringConfig::new(260.0, 26.0, 1.0);

/// Calculates responsive grid positions, actual item width and total container height.
/// Distributes items evenly into `cols` columns where each column expands to fill available width.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn compute_responsive_grid_layout(
    available_width: Pixels,
    min_item_width: Pixels,
    item_height: Pixels,
    gap: Pixels,
    count: usize,
) -> (Vec<Point<Pixels>>, Pixels, Pixels) {
    if count == 0 {
        return (Vec::new(), available_width, px(0.0));
    }

    let min_w = min_item_width.max(px(100.0));
    let cols = (((available_width + gap) / (min_w + gap)).floor() as usize).max(1);
    let cols_f = cols as f32;
    let total_gaps = gap * (cols.saturating_sub(1)) as f32;
    let actual_item_width = ((available_width - total_gaps) / cols_f).max(min_w);

    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        let x = (actual_item_width + gap) * col as f32;
        let y = (item_height + gap) * row as f32;
        positions.push(point(x, y));
    }

    let total_rows = count.div_ceil(cols);
    let total_height =
        item_height * total_rows as f32 + gap * (total_rows.saturating_sub(1)) as f32;

    (positions, actual_item_width, total_height)
}

/// Renders items in an animated flow grid with spring physics for positional transitions.
pub fn render_animated_grid(
    grid_id: &'static str,
    available_width: Pixels,
    min_item_width: Pixels,
    item_height: Pixels,
    gap: Pixels,
    items: Vec<(&'static str, AnyElement)>,
) -> impl IntoElement {
    let (positions, item_width, total_height) = compute_responsive_grid_layout(
        available_width,
        min_item_width,
        item_height,
        gap,
        items.len(),
    );

    let mut grid_el = div()
        .id(ElementId::Name(grid_id.into()))
        .relative()
        .w_full()
        .h(total_height)
        .with_spring(
            ElementId::Name(format!("{grid_id}_spring_h").into()),
            SpringAnimation::new(GRID_LAYOUT_SPRING).to(f32::from(total_height)),
            |grid, h| grid.h(px(h)),
        );

    for (i, (card_id, card_el)) in items.into_iter().enumerate() {
        let pos = positions.get(i).copied().unwrap_or(point(px(0.0), px(0.0)));
        grid_el = grid_el.child(
            div()
                .absolute()
                .left(pos.x)
                .with_spring(
                    ElementId::Name(format!("{card_id}_grid_x").into()),
                    SpringAnimation::new(GRID_LAYOUT_SPRING).to(f32::from(pos.x)),
                    |card, x| card.left(px(x)),
                )
                .child(
                    div()
                        .top(pos.y)
                        .with_spring(
                            ElementId::Name(format!("{card_id}_grid_y").into()),
                            SpringAnimation::new(GRID_LAYOUT_SPRING).to(f32::from(pos.y)),
                            |card, y| card.top(px(y)),
                        )
                        .child(
                            div()
                                .w(item_width)
                                .h(item_height)
                                .with_spring(
                                    ElementId::Name(format!("{card_id}_grid_w").into()),
                                    SpringAnimation::new(GRID_LAYOUT_SPRING)
                                        .to(f32::from(item_width)),
                                    |card, w| card.w(px(w)),
                                )
                                .child(card_el),
                        ),
                ),
        );
    }

    grid_el
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let (pos, w, h) =
            compute_responsive_grid_layout(px(600.0), px(300.0), px(60.0), px(12.0), 0);
        assert!(pos.is_empty());
        assert_eq!(h, px(0.0));
        assert_eq!(w, px(600.0));
    }

    #[test]
    fn test_two_columns_layout() {
        // available = 652, min_w = 320, gap = 12
        // cols = floor((652 + 12) / (320 + 12)) = floor(664 / 332) = 2
        // actual_item_width = (652 - 12) / 2 = 320
        let (pos, item_w, total_h) =
            compute_responsive_grid_layout(px(652.0), px(320.0), px(58.0), px(12.0), 4);
        assert_eq!(pos.len(), 4);
        assert_eq!(item_w, px(320.0));
        assert_eq!(pos[0], point(px(0.0), px(0.0)));
        assert_eq!(pos[1], point(px(332.0), px(0.0)));
        assert_eq!(pos[2], point(px(0.0), px(70.0)));
        assert_eq!(pos[3], point(px(332.0), px(70.0)));
        assert_eq!(total_h, px(128.0)); // 58*2 + 12
    }

    #[test]
    fn test_single_column_when_narrow() {
        let (pos, item_w, total_h) =
            compute_responsive_grid_layout(px(340.0), px(320.0), px(58.0), px(12.0), 2);
        assert_eq!(pos.len(), 2);
        assert_eq!(item_w, px(340.0));
        assert_eq!(pos[0], point(px(0.0), px(0.0)));
        assert_eq!(pos[1], point(px(0.0), px(70.0)));
        assert_eq!(total_h, px(128.0));
    }
}
