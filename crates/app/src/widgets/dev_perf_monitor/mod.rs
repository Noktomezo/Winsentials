use std::sync::Arc;

use gpui::{
    App, IntoElement, Pixels, Point, RenderOnce, Window,
};

use crate::features::navigation::AppRoute;
use crate::shared::theme::Theme;

pub mod inspector;
pub mod render_expanded;
pub mod render_minimized;
pub mod state;

#[cfg(test)]
mod tests;

pub use state::*;
use render_expanded::render_expanded_monitor;
use render_minimized::render_minimized_monitor;

#[derive(IntoElement)]
pub struct DevPerfMonitor {
    snapshot: DevPerfSnapshot,
    current_route: AppRoute,
    on_toggle_minimize: DevActionCallback,
    on_toggle_freeze_telemetry: DevActionCallback,
    on_toggle_chart_anim: DevActionCallback,
    on_toggle_continuous: DevActionCallback,
    on_start_drag: DevDragCallback,
    on_drag_move: DevDragMoveCallback,
    on_end_drag: DevActionCallback,
    on_close: DevActionCallback,
    on_hover_control: Option<HoverControlHandler>,
}

impl DevPerfMonitor {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        snapshot: DevPerfSnapshot,
        current_route: AppRoute,
        on_toggle_minimize: impl Fn(&mut Window, &mut App) + 'static,
        on_toggle_freeze_telemetry: impl Fn(&mut Window, &mut App) + 'static,
        on_toggle_chart_anim: impl Fn(&mut Window, &mut App) + 'static,
        on_toggle_continuous: impl Fn(&mut Window, &mut App) + 'static,
        on_start_drag: impl Fn(Point<Pixels>, Point<Pixels>, &mut Window, &mut App) + 'static,
        on_drag_move: impl Fn(Point<Pixels>, bool, &mut Window, &mut App) + 'static,
        on_end_drag: impl Fn(&mut Window, &mut App) + 'static,
        on_close: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            snapshot,
            current_route,
            on_toggle_minimize: Arc::new(on_toggle_minimize),
            on_toggle_freeze_telemetry: Arc::new(on_toggle_freeze_telemetry),
            on_toggle_chart_anim: Arc::new(on_toggle_chart_anim),
            on_toggle_continuous: Arc::new(on_toggle_continuous),
            on_start_drag: Arc::new(on_start_drag),
            on_drag_move: Arc::new(on_drag_move),
            on_end_drag: Arc::new(on_end_drag),
            on_close: Arc::new(on_close),
            on_hover_control: None,
        }
    }

    #[must_use]
    pub fn on_hover_control(
        mut self,
        handler: impl Fn(&'static str, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_control = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for DevPerfMonitor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let viewport = window.viewport_size();
        let current_pos = self.snapshot.current_pos(viewport.width, viewport.height);
        let status_color = fps_status_color(self.snapshot.displayed_fps);

        let fps_text = if self.snapshot.displayed_fps <= 0.0 {
            "IDLE".to_string()
        } else {
            format!("{:.0} FPS", self.snapshot.displayed_fps)
        };

        if self.snapshot.minimized {
            render_minimized_monitor(
                &self.snapshot,
                current_pos,
                status_color,
                &fps_text,
                &self.on_toggle_minimize,
                &self.on_start_drag,
                &self.on_drag_move,
                &self.on_end_drag,
                &self.on_hover_control,
                &theme,
            ).into_any_element()
        } else {
            render_expanded_monitor(
                &self.snapshot,
                self.current_route,
                current_pos,
                status_color,
                &self.on_toggle_minimize,
                &self.on_close,
                &self.on_start_drag,
                &self.on_drag_move,
                &self.on_end_drag,
                &self.on_toggle_freeze_telemetry,
                &self.on_toggle_chart_anim,
                &self.on_toggle_continuous,
                &self.on_hover_control,
                &theme,
            ).into_any_element()
        }
    }
}