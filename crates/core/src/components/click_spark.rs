use std::f32::consts::PI;
use std::time::{Duration, Instant};

use gpui::{
    Animation, AnimationExt, ElementId, InteractiveElement, IntoElement, ParentElement,
    PathBuilder, Pixels, Point, Rgba, Styled, Window, canvas, div, point, px,
};

pub const SPARK_DURATION: Duration = Duration::from_millis(350);
pub const SPARK_COUNT: usize = 8;
pub const SPARK_RADIUS: f32 = 18.0;
pub const SPARK_SIZE: f32 = 10.0;
pub const SPARK_LINE_WIDTH: f32 = 2.0;

#[derive(Clone, Copy, Debug)]
pub struct ClickSparkBurst {
    pub id: usize,
    pub origin: Point<Pixels>,
    pub start_time: Instant,
}

impl ClickSparkBurst {
    #[must_use]
    pub fn new(id: usize, origin: Point<Pixels>) -> Self {
        Self {
            id,
            origin,
            start_time: Instant::now(),
        }
    }

    #[must_use]
    pub fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start_time).as_secs_f32();
        let total = SPARK_DURATION.as_secs_f32();
        (elapsed / total).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn is_alive(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start_time) < SPARK_DURATION
    }
}

#[must_use]
pub fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

#[must_use]
pub fn spark_ray_segment(
    origin: Point<Pixels>,
    angle: f32,
    progress: f32,
) -> (Point<Pixels>, Point<Pixels>) {
    let eased = ease_out(progress);
    let distance = eased * SPARK_RADIUS;
    let line_length = SPARK_SIZE * (1.0 - eased);

    let cos = angle.cos();
    let sin = angle.sin();

    let x0 = f32::from(origin.x);
    let y0 = f32::from(origin.y);

    let x1 = x0 + distance * cos;
    let y1 = y0 + distance * sin;
    let x2 = x0 + (distance + line_length) * cos;
    let y2 = y0 + (distance + line_length) * sin;

    (point(px(x1), px(y1)), point(px(x2), px(y2)))
}

pub fn render_click_sparks(
    sparks: &[ClickSparkBurst],
    spark_color: Rgba,
    now: Instant,
) -> impl IntoElement {
    let active_sparks: Vec<ClickSparkBurst> =
        sparks.iter().copied().filter(|s| s.is_alive(now)).collect();

    let latest_id = active_sparks.last().map_or(0, |s| s.id);

    div()
        .id("click_sparks_overlay")
        .absolute()
        .size_full()
        .with_animation(
            ElementId::Name(format!("sparks_tick_{latest_id}").into()),
            Animation::new(SPARK_DURATION),
            move |overlay, _delta| {
                let sparks = active_sparks.clone();
                overlay.child(canvas(
                    move |_bounds, _window, _cx| {},
                    move |_bounds, (), window, _cx| {
                        let render_now = Instant::now();
                        for spark in &sparks {
                            draw_spark_burst(window, spark, spark_color, render_now);
                        }
                    },
                ))
            },
        )
}

fn draw_spark_burst(window: &mut Window, spark: &ClickSparkBurst, spark_color: Rgba, now: Instant) {
    let progress = spark.progress(now);
    if progress >= 1.0 {
        return;
    }

    let alpha = (1.0 - progress).clamp(0.0, 1.0);
    let color = spark_color.opacity(alpha);

    for i in 0..SPARK_COUNT {
        #[allow(clippy::cast_precision_loss)]
        let angle = (2.0 * PI * i as f32) / (SPARK_COUNT as f32);
        let (start, end) = spark_ray_segment(spark.origin, angle, progress);

        let mut builder = PathBuilder::stroke(px(SPARK_LINE_WIDTH));
        builder.move_to(start);
        builder.line_to(end);

        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_out_bounds() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.5);
    }

    #[test]
    fn test_spark_progress_lifecycle() {
        let burst = ClickSparkBurst::new(1, point(px(100.0), px(200.0)));
        assert!(burst.is_alive(burst.start_time));
        assert_eq!(burst.progress(burst.start_time), 0.0);

        let half_time = burst.start_time + SPARK_DURATION / 2;
        assert!(burst.is_alive(half_time));
        let p = burst.progress(half_time);
        assert!((p - 0.5).abs() < 0.01);

        let end_time = burst.start_time + SPARK_DURATION + Duration::from_millis(10);
        assert!(!burst.is_alive(end_time));
        assert_eq!(burst.progress(end_time), 1.0);
    }

    #[test]
    fn test_spark_ray_segment_geometry() {
        let origin = point(px(50.0), px(50.0));
        let (start_0, end_0) = spark_ray_segment(origin, 0.0, 0.0);
        // At progress 0: distance is 0, line_length is SPARK_SIZE (10.0)
        assert_eq!(start_0.x, px(50.0));
        assert_eq!(start_0.y, px(50.0));
        assert_eq!(end_0.x, px(60.0));
        assert_eq!(end_0.y, px(50.0));

        let (start_1, end_1) = spark_ray_segment(origin, 0.0, 1.0);
        // At progress 1: distance is SPARK_RADIUS (18.0), line_length is 0
        assert_eq!(start_1.x, px(68.0));
        assert_eq!(start_1.y, px(50.0));
        assert_eq!(end_1.x, px(68.0));
        assert_eq!(end_1.y, px(50.0));
    }
}
