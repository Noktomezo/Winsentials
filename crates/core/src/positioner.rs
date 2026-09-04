use gpui::{Bounds, Pixels, Point, px};

/// Preferred side for placing an anchored overlay relative to its trigger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Placement {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

impl Placement {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Bottom => Self::Top,
            Self::Top => Self::Bottom,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }
}

/// Alignment of the overlay along the placement side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

/// Positioning strategy: either anchored to a trigger's bounds or to a point (cursor).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionerStrategy {
    /// Anchored to a trigger's bounding box (e.g. dropdown, menu, popover).
    Side {
        trigger_bounds: Bounds<Pixels>,
        preferred_placement: Placement,
        align: Align,
        offset: Pixels,
    },
    /// Anchored to a cursor/pointer coordinate (e.g. cursor tooltip).
    Cursor {
        cursor_pos: Point<Pixels>,
        preferred_placement: Placement,
        offset: Pixels,
    },
}

/// Resolved popup coordinates and the resulting placement side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedPosition {
    pub origin: Point<Pixels>,
    pub placement: Placement,
}

/// Headless positioner, automatic side-flipping, and viewport clamping engine.
///
/// Designed after the Base UI / Radix anchored positioner pattern:
/// - Evaluates available viewport space in both directions
/// - Automatically flips to the opposite side if space is constrained
/// - Aligns along the trigger axis (`Start`, `Center`, `End`)
/// - Strictly clamps within window viewport boundaries and titlebar margins
#[derive(Clone, Debug)]
pub struct Positioner {
    pub strategy: PositionerStrategy,
    pub margin: Pixels,
    pub top_margin: Pixels,
}

impl Positioner {
    #[must_use]
    pub fn side(trigger_bounds: Bounds<Pixels>) -> Self {
        Self {
            strategy: PositionerStrategy::Side {
                trigger_bounds,
                preferred_placement: Placement::Bottom,
                align: Align::Start,
                offset: px(4.0),
            },
            margin: px(8.0),
            top_margin: px(36.0),
        }
    }

    #[must_use]
    pub fn cursor(cursor_pos: Point<Pixels>) -> Self {
        Self {
            strategy: PositionerStrategy::Cursor {
                cursor_pos,
                preferred_placement: Placement::Top,
                offset: px(8.0),
            },
            margin: px(8.0),
            top_margin: px(36.0),
        }
    }

    #[must_use]
    pub fn placement(mut self, placement: Placement) -> Self {
        match &mut self.strategy {
            PositionerStrategy::Side {
                preferred_placement,
                ..
            }
            | PositionerStrategy::Cursor {
                preferred_placement,
                ..
            } => *preferred_placement = placement,
        }
        self
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        if let PositionerStrategy::Side { align: a, .. } = &mut self.strategy {
            *a = align;
        }
        self
    }

    #[must_use]
    pub fn offset(mut self, offset: Pixels) -> Self {
        match &mut self.strategy {
            PositionerStrategy::Side { offset: o, .. }
            | PositionerStrategy::Cursor { offset: o, .. } => *o = offset,
        }
        self
    }

    #[must_use]
    pub fn margin(mut self, margin: Pixels) -> Self {
        self.margin = margin;
        self
    }

    #[must_use]
    pub fn top_margin(mut self, top_margin: Pixels) -> Self {
        self.top_margin = top_margin;
        self
    }

    /// Resolves the absolute coordinates `(x, y)` for content of size `(content_width, content_height)`
    /// inside a window with `(viewport_width, viewport_height)`.
    #[must_use]
    pub fn resolve(
        &self,
        content_width: Pixels,
        content_height: Pixels,
        viewport_width: Pixels,
        viewport_height: Pixels,
    ) -> ResolvedPosition {
        match self.strategy {
            PositionerStrategy::Side {
                trigger_bounds,
                preferred_placement,
                align,
                offset,
            } => {
                let (resolved_y, placement) = if preferred_placement.is_vertical() {
                    let space_below =
                        viewport_height - self.margin - (trigger_bounds.bottom() + offset);
                    let space_above = (trigger_bounds.top() - offset) - self.top_margin;

                    let fits_below = space_below >= content_height;
                    let fits_above = space_above >= content_height;

                    let (y, actual_side) = match preferred_placement {
                        Placement::Bottom => {
                            if fits_below || space_below >= space_above {
                                (trigger_bounds.bottom() + offset, Placement::Bottom)
                            } else {
                                (
                                    trigger_bounds.top() - offset - content_height,
                                    Placement::Top,
                                )
                            }
                        }
                        Placement::Top => {
                            if fits_above || space_above >= space_below {
                                (
                                    trigger_bounds.top() - offset - content_height,
                                    Placement::Top,
                                )
                            } else {
                                (trigger_bounds.bottom() + offset, Placement::Bottom)
                            }
                        }
                        _ => unreachable!(),
                    };

                    let min_y = self.top_margin;
                    let max_y = (viewport_height - content_height - self.margin).max(min_y);
                    (y.clamp(min_y, max_y), actual_side)
                } else {
                    let y = trigger_bounds.top();
                    let min_y = self.top_margin;
                    let max_y = (viewport_height - content_height - self.margin).max(min_y);
                    (y.clamp(min_y, max_y), preferred_placement)
                };

                let preferred_x = match align {
                    Align::Start => trigger_bounds.left(),
                    Align::Center => {
                        trigger_bounds.left() + (trigger_bounds.size.width - content_width) / 2.0
                    }
                    Align::End => trigger_bounds.right() - content_width,
                };

                let min_x = self.margin;
                let max_x = (viewport_width - content_width - self.margin).max(min_x);
                let resolved_x = preferred_x.clamp(min_x, max_x);

                ResolvedPosition {
                    origin: gpui::point(resolved_x, resolved_y),
                    placement,
                }
            }
            PositionerStrategy::Cursor {
                cursor_pos,
                preferred_placement: _,
                offset,
            } => {
                let effective_top = if cursor_pos.y < self.top_margin {
                    self.margin
                } else {
                    self.top_margin
                };

                let space_above = cursor_pos.y - offset - effective_top;
                let space_below = viewport_height - self.margin - (cursor_pos.y + px(20.0));

                let (y, placement) = if space_above >= content_height {
                    (cursor_pos.y - content_height - offset, Placement::Top)
                } else if space_below >= content_height {
                    (cursor_pos.y + px(20.0), Placement::Bottom)
                } else if space_above >= space_below {
                    (cursor_pos.y - content_height - offset, Placement::Top)
                } else {
                    (cursor_pos.y + px(20.0), Placement::Bottom)
                };

                let min_y = effective_top;
                let max_y = (viewport_height - content_height - self.margin).max(min_y);
                let resolved_y = y.clamp(min_y, max_y);

                let offset_x = px(12.0);
                let preferred_x =
                    if cursor_pos.x + offset_x + content_width > viewport_width - self.margin {
                        cursor_pos.x - content_width - offset_x
                    } else {
                        cursor_pos.x + offset_x
                    };

                let min_x = self.margin;
                let max_x = (viewport_width - content_width - self.margin).max(min_x);
                let resolved_x = preferred_x.clamp(min_x, max_x);

                ResolvedPosition {
                    origin: gpui::point(resolved_x, resolved_y),
                    placement,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};

    #[test]
    fn test_side_positioner_fits_below() {
        let trigger = Bounds {
            origin: point(px(100.0), px(200.0)),
            size: size(px(120.0), px(32.0)),
        };
        let pos = Positioner::side(trigger)
            .placement(Placement::Bottom)
            .align(Align::Start)
            .offset(px(4.0));

        let res = pos.resolve(px(120.0), px(100.0), px(800.0), px(600.0));
        assert_eq!(res.placement, Placement::Bottom);
        assert_eq!(res.origin.x, px(100.0));
        assert_eq!(res.origin.y, px(236.0)); // 200 + 32 + 4
    }

    #[test]
    fn test_side_positioner_flips_upward_when_space_below_is_insufficient() {
        let trigger = Bounds {
            origin: point(px(100.0), px(550.0)),
            size: size(px(120.0), px(32.0)),
        };
        let pos = Positioner::side(trigger)
            .placement(Placement::Bottom)
            .align(Align::Start)
            .offset(px(4.0));

        let res = pos.resolve(px(120.0), px(100.0), px(800.0), px(600.0));
        assert_eq!(res.placement, Placement::Top);
        assert_eq!(res.origin.x, px(100.0));
        assert_eq!(res.origin.y, px(446.0)); // 550 - 4 - 100
    }

    #[test]
    fn test_cursor_positioner_flips_and_clamps() {
        let pos = Positioner::cursor(point(px(400.0), px(580.0)));
        let res = pos.resolve(px(320.0), px(140.0), px(800.0), px(600.0));

        assert_eq!(res.placement, Placement::Top);
        assert!(res.origin.y >= px(36.0));
        assert!(res.origin.y + px(140.0) <= px(592.0));
    }
}
