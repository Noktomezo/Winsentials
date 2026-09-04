use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    App, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce,
    Rgba, StatefulInteractiveElement, Styled, Window, div, point, px, rgba,
};

use crate::features::navigation::AppRoute;
use crate::shared::motion::hover_spring;
use crate::shared::theme::Theme;
use crate::shared::ui::history_graph::{HistoryGraphPalette, render_stepped_history_graph_sized};
use crate::shared::ui::{Chip, IconButton};

pub const BOUNDS_PADDING: f32 = 16.0;
pub const TITLEBAR_HEIGHT: f32 = 44.0;
pub const MINIMIZED_WIDTH: f32 = 156.0;
pub const MINIMIZED_HEIGHT: f32 = 32.0;
pub const EXPANDED_WIDTH: f32 = 324.0;
pub const EXPANDED_HEIGHT: f32 = 380.0;
pub const MAX_SAMPLES: usize = 60;
pub const FRAME_BUDGET_60HZ: f32 = 16.667;
pub const FPS_TOLERANCE: f32 = 0.95; // 5% vsync tolerance (57-60 fps is healthy 60Hz)
pub const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(400);

pub type DevActionCallback = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type HoverControlHandler = Arc<dyn Fn(&'static str, bool, &mut Window, &mut App) + 'static>;
pub type DevDragCallback =
    Arc<dyn Fn(Point<Pixels>, Point<Pixels>, &mut Window, &mut App) + 'static>;
pub type DevDragMoveCallback = Arc<dyn Fn(Point<Pixels>, bool, &mut Window, &mut App) + 'static>;

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct DevPerfMonitorState {
    pub enabled: bool,
    pub minimized: bool,
    pub position: Option<Point<Pixels>>,
    pub is_dragging: bool,
    pub drag_start_mouse: Point<Pixels>,
    pub drag_start_pos: Point<Pixels>,

    // Per-frame actual frame time (frametime interval dt) in milliseconds
    pub frame_times: VecDeque<f32>,
    pub frame_samples_cache: Arc<[f32]>,
    pub current_frame_ms: f32,
    pub last_frame_instant: Instant,

    // CPU execution time (render() virtual DOM duration)
    pub cpu_draw_ms: f32,

    // Timestamps of presented frames (for rolling FPS calculation)
    pub present_instants: VecDeque<Instant>,

    // Displayed metrics (decoupled from render loop, refreshed every 400ms to avoid digit jitter)
    pub last_stats_publish: Instant,
    pub displayed_fps: f32,
    pub displayed_frame_ms: f32,
    pub displayed_p95_ms: f32,
    pub displayed_drop_rate: f32,
    pub displayed_memory_mb: f32,

    // Dev overrides / toggles for isolating bottlenecks
    pub freeze_telemetry: bool,
    pub disable_chart_animation: bool,
    pub continuous_mode: bool,
    pub inspect_expanded: bool,
    pub hovered_control: Option<&'static str>,
}

impl Default for DevPerfMonitorState {
    fn default() -> Self {
        Self::new()
    }
}

impl DevPerfMonitorState {
    #[must_use]
    pub fn new() -> Self {
        let now = Instant::now();
        let mut frame_times = VecDeque::with_capacity(MAX_SAMPLES);
        for _ in 0..MAX_SAMPLES {
            frame_times.push_back(16.6);
        }
        let frame_samples_cache = Arc::from(frame_times.iter().copied().collect::<Vec<_>>());

        Self {
            enabled: true,
            minimized: true,
            position: None,
            is_dragging: false,
            drag_start_mouse: point(px(0.0), px(0.0)),
            drag_start_pos: point(px(0.0), px(0.0)),
            frame_times,
            frame_samples_cache,
            last_frame_instant: now,
            current_frame_ms: 16.6,
            cpu_draw_ms: 0.5,
            present_instants: VecDeque::with_capacity(MAX_SAMPLES),
            last_stats_publish: now,
            displayed_fps: 60.0,
            displayed_frame_ms: 16.6,
            displayed_p95_ms: 16.6,
            displayed_drop_rate: 0.0,
            displayed_memory_mb: get_process_memory_mb().unwrap_or(0.0),
            freeze_telemetry: false,
            disable_chart_animation: false,
            // Continuous mode is ON by default (like gpui-component/fps) so frame trace
            // advances steadily at native VSync rate instead of halting on mouse idle
            continuous_mode: true,
            inspect_expanded: true,
            hovered_control: None,
        }
    }

    pub fn set_hovered_control(&mut self, ctrl: Option<&'static str>) {
        self.hovered_control = ctrl;
    }

    pub fn record_frame(&mut self, cpu_draw_ms: f32) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_instant).as_secs_f32() * 1000.0;
        self.last_frame_instant = now;
        self.cpu_draw_ms = cpu_draw_ms;

        let sample_ms = if (0.5..100.0).contains(&dt) {
            dt
        } else {
            cpu_draw_ms.clamp(1.0, 33.3)
        };
        self.current_frame_ms = sample_ms;
        if self.frame_times.len() >= MAX_SAMPLES {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(sample_ms);
        self.frame_samples_cache = Arc::from(self.frame_times.iter().copied().collect::<Vec<_>>());

        if (0.5..100.0).contains(&dt) {
            if self.present_instants.len() >= MAX_SAMPLES {
                self.present_instants.pop_front();
            }
            self.present_instants.push_back(now);
        }

        // Best practice: decouple readout republishing (~400ms cadence)
        // Recomputed per frame they flicker through digits too fast to read.
        if now.duration_since(self.last_stats_publish) >= DISPLAY_REFRESH_INTERVAL {
            self.last_stats_publish = now;

            // 1. Calculate FPS from present timestamps window
            let fps = if self.present_instants.len() >= 2 {
                let oldest = *self.present_instants.front().expect("has elements");
                let newest = *self.present_instants.back().expect("has elements");
                let span = newest.duration_since(oldest).as_secs_f32();
                if span > 0.0 && now.duration_since(newest).as_millis() <= 350 {
                    #[allow(clippy::cast_precision_loss)]
                    let rate = (self.present_instants.len() - 1) as f32 / span;
                    rate
                } else {
                    0.0
                }
            } else {
                0.0
            };
            self.displayed_fps = fps;

            // 2. Mean frame duration across retained frames
            #[allow(clippy::cast_precision_loss)]
            let count = self.frame_times.len() as f32;
            let mean_ms = if count > 0.0 {
                self.frame_times.iter().sum::<f32>() / count
            } else if fps > 0.0 {
                1000.0 / fps
            } else {
                16.6
            };
            self.displayed_frame_ms = mean_ms;

            // 3. P95 tail latency (95% of frames come in under this duration)
            let mut sorted: Vec<f32> = self.frame_times.iter().copied().collect();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let p95_idx =
                ((sorted.len() as f32 * 0.95).floor() as usize).min(sorted.len().saturating_sub(1));
            self.displayed_p95_ms = sorted.get(p95_idx).copied().unwrap_or(mean_ms);

            // 4. Drop rate (% of frames where actual frame duration exceeded budget 16.6ms by 5%)
            let dropped = self
                .frame_times
                .iter()
                .filter(|&&d| d > (FRAME_BUDGET_60HZ * 1.05))
                .count();
            #[allow(clippy::cast_precision_loss)]
            let drop_pct = if count > 0.0 {
                (dropped as f32 / count) * 100.0
            } else {
                0.0
            };
            self.displayed_drop_rate = drop_pct;

            // 5. Memory commit size (PrivateUsage)
            if let Some(mem) = get_process_memory_mb() {
                self.displayed_memory_mb = mem;
            }
        }
    }

    #[must_use]
    pub fn current_pos(&self, viewport_width: Pixels, viewport_height: Pixels) -> Point<Pixels> {
        let (width, height) = if self.minimized {
            (MINIMIZED_WIDTH, MINIMIZED_HEIGHT)
        } else {
            (EXPANDED_WIDTH, EXPANDED_HEIGHT)
        };

        let min_x = px(BOUNDS_PADDING);
        let max_x = (viewport_width - px(width) - px(BOUNDS_PADDING)).max(min_x);
        let min_y = px(TITLEBAR_HEIGHT);
        let max_y = (viewport_height - px(height) - px(BOUNDS_PADDING)).max(min_y);

        self.position.map_or_else(
            || point(max_x, min_y + px(12.0)),
            |pos| point(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y)),
        )
    }

    pub fn start_drag(&mut self, mouse_pos: Point<Pixels>, current_widget_pos: Point<Pixels>) {
        self.is_dragging = true;
        self.drag_start_mouse = mouse_pos;
        self.drag_start_pos = current_widget_pos;
    }

    pub fn update_drag(
        &mut self,
        mouse_pos: Point<Pixels>,
        viewport_width: Pixels,
        viewport_height: Pixels,
    ) {
        if !self.is_dragging {
            return;
        }

        let delta_x = mouse_pos.x - self.drag_start_mouse.x;
        let delta_y = mouse_pos.y - self.drag_start_mouse.y;

        let (width, height) = if self.minimized {
            (MINIMIZED_WIDTH, MINIMIZED_HEIGHT)
        } else {
            (EXPANDED_WIDTH, EXPANDED_HEIGHT)
        };

        let min_x = px(BOUNDS_PADDING);
        let max_x = (viewport_width - px(width) - px(BOUNDS_PADDING)).max(min_x);
        let min_y = px(TITLEBAR_HEIGHT);
        let max_y = (viewport_height - px(height) - px(BOUNDS_PADDING)).max(min_y);

        let new_x = (self.drag_start_pos.x + delta_x).clamp(min_x, max_x);
        let new_y = (self.drag_start_pos.y + delta_y).clamp(min_y, max_y);

        self.position = Some(point(new_x, new_y));
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }

    #[must_use]
    pub fn snapshot(&self) -> DevPerfSnapshot {
        DevPerfSnapshot {
            enabled: self.enabled,
            minimized: self.minimized,
            position: self.position,
            displayed_fps: self.displayed_fps,
            displayed_frame_ms: self.displayed_frame_ms,
            displayed_p95_ms: self.displayed_p95_ms,
            displayed_drop_rate: self.displayed_drop_rate,
            displayed_memory_mb: self.displayed_memory_mb,
            cpu_draw_ms: self.cpu_draw_ms,
            freeze_telemetry: self.freeze_telemetry,
            disable_chart_animation: self.disable_chart_animation,
            continuous_mode: self.continuous_mode,
            is_dragging: self.is_dragging,
            frame_samples: Arc::clone(&self.frame_samples_cache),
            hovered_control: self.hovered_control,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct DevPerfSnapshot {
    pub enabled: bool,
    pub minimized: bool,
    pub position: Option<Point<Pixels>>,
    pub displayed_fps: f32,
    pub displayed_frame_ms: f32,
    pub displayed_p95_ms: f32,
    pub displayed_drop_rate: f32,
    pub displayed_memory_mb: f32,
    pub cpu_draw_ms: f32,
    pub freeze_telemetry: bool,
    pub disable_chart_animation: bool,
    pub continuous_mode: bool,
    pub is_dragging: bool,
    pub frame_samples: Arc<[f32]>,
    pub hovered_control: Option<&'static str>,
}

impl DevPerfSnapshot {
    #[must_use]
    pub fn current_pos(&self, viewport_width: Pixels, viewport_height: Pixels) -> Point<Pixels> {
        let (width, height) = if self.minimized {
            (MINIMIZED_WIDTH, MINIMIZED_HEIGHT)
        } else {
            (EXPANDED_WIDTH, EXPANDED_HEIGHT)
        };

        let min_x = px(BOUNDS_PADDING);
        let max_x = (viewport_width - px(width) - px(BOUNDS_PADDING)).max(min_x);
        let min_y = px(TITLEBAR_HEIGHT);
        let max_y = (viewport_height - px(height) - px(BOUNDS_PADDING)).max(min_y);

        self.position.map_or_else(
            || point(max_x, min_y + px(12.0)),
            |pos| point(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y)),
        )
    }
}

/// Reads `PrivateUsage` (Private Commit) via `GetProcessMemoryInfo`:
/// This matches Windows Task Manager's private commit column rather than generic `WorkingSet`,
/// excluding read-only shared DLL pages of DirectX/graphics driver runtimes.
#[cfg(target_os = "windows")]
#[allow(unsafe_code, clippy::cast_possible_truncation)]
fn get_process_memory_mb() -> Option<f32> {
    use std::mem::size_of;
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let size = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        counters.cb = size;
        let handle = GetCurrentProcess();
        if GetProcessMemoryInfo(
            handle,
            (&raw mut counters).cast::<PROCESS_MEMORY_COUNTERS>(),
            size,
        ) != 0
        {
            #[allow(clippy::cast_precision_loss)]
            Some(counters.PrivateUsage as f32 / (1024.0 * 1024.0))
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_process_memory_mb() -> Option<f32> {
    None
}

/// Best practice: Grades FPS against the target rate with a 5% vsync tolerance.
/// A healthy 60Hz display lands at 58-60 fps, so exact comparison would falsely flag it.
#[must_use]
fn fps_status_color(fps: f32) -> Rgba {
    if fps <= 0.0 {
        rgba(0x8888_88FF) // Muted gray when idle
    } else if fps >= 60.0 * FPS_TOLERANCE {
        rgba(0x10B9_81FF) // Emerald green (healthy)
    } else if fps >= 30.0 {
        rgba(0xF59E_0BFF) // Amber (warning)
    } else {
        rgba(0xEF44_44FF) // Crimson red (sub-30 fps)
    }
}

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
    #[allow(clippy::too_many_lines)]
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

        let on_hover_control = self.on_hover_control;

        if self.snapshot.minimized {
            let on_toggle = self.on_toggle_minimize.clone();
            let on_start_drag = self.on_start_drag.clone();
            let on_drag_move = self.on_drag_move.clone();
            let end_drag_action = self.on_end_drag.clone();
            let pos_clone = current_pos;
            let is_dragging = self.snapshot.is_dragging;

            let is_expand_hovered = self.snapshot.hovered_control == Some("expand");
            let expand_spring = hover_spring(if is_expand_hovered { 0.5 } else { 0.0 });
            let on_hover_expand = on_hover_control.clone();

            div()
                .id("dev_perf_monitor_minimized")
                .absolute()
                .left(current_pos.x)
                .top(current_pos.y)
                .w(px(MINIMIZED_WIDTH))
                .h(px(MINIMIZED_HEIGHT))
                .rounded(px(8.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .flex()
                .items_center()
                .justify_between()
                .px(px(10.0))
                .cursor_move()
                .on_mouse_down(MouseButton::Left, {
                    let end_cb = end_drag_action.clone();
                    move |event, window, cx| {
                        cx.stop_propagation();
                        if is_dragging {
                            end_cb(window, cx);
                        } else {
                            on_start_drag(event.position, pos_clone, window, cx);
                        }
                    }
                })
                .on_mouse_down(MouseButton::Right, {
                    let end_cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        cx.stop_propagation();
                        if is_dragging {
                            end_cb(window, cx);
                        }
                    }
                })
                .on_mouse_move(move |event, window, cx| {
                    if event.dragging() {
                        cx.stop_propagation();
                        on_drag_move(event.position, true, window, cx);
                    } else if is_dragging {
                        cx.stop_propagation();
                        on_drag_move(event.position, false, window, cx);
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        cx.stop_propagation();
                        cb(window, cx);
                    }
                })
                .on_mouse_up_out(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cb(window, cx);
                        }
                    }
                })
                .on_mouse_up(MouseButton::Right, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        cx.stop_propagation();
                        cb(window, cx);
                    }
                })
                .on_mouse_up_out(MouseButton::Right, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cb(window, cx);
                        }
                    }
                })
                .on_click(|_, _, cx| cx.stop_propagation())
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().size(px(8.0)).rounded_full().bg(status_color))
                        .child(
                            div()
                                .font_family("Consolas")
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(fps_text),
                        )
                        .child(
                            div()
                                .font_family("Consolas")
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.text_muted)
                                .child(format!("{:.1}ms", self.snapshot.displayed_frame_ms)),
                        ),
                )
                .child(
                    IconButton::new("dev_perf_expand_btn", "icons/maximize-2.svg")
                        .button_size(px(22.0))
                        .icon_size(px(13.0))
                        .spring(expand_spring, theme.accent_cyan)
                        .on_hover(move |hov, window, cx| {
                            if let Some(ref h) = on_hover_expand {
                                h("expand", hov, window, cx);
                            }
                        })
                        .on_mouse_down(move |window, cx| {
                            on_toggle(window, cx);
                        }),
                )
        } else {
            let on_toggle = self.on_toggle_minimize.clone();
            let on_close = self.on_close.clone();
            let on_start_drag = self.on_start_drag.clone();
            let on_drag_move = self.on_drag_move.clone();
            let end_drag_action = self.on_end_drag.clone();
            let on_freeze_telemetry = self.on_toggle_freeze_telemetry.clone();
            let on_chart_anim = self.on_toggle_chart_anim.clone();
            let on_continuous = self.on_toggle_continuous.clone();
            let pos_clone = current_pos;
            let is_dragging = self.snapshot.is_dragging;

            let route_weight = match self.current_route {
                AppRoute::CpuDetail
                | AppRoute::RamDetail
                | AppRoute::DiskDetail(_)
                | AppRoute::NetworkDetail(_)
                | AppRoute::GpuDetail(_) => "High (Glide)",
                AppRoute::ContextMenu => "Medium (Cards)",
                AppRoute::Cleanup => "High (Scan)",
                _ => "Normal",
            };

            let ram_str = format!("{:.1} MB", self.snapshot.displayed_memory_mb);
            let frame_samples = Arc::clone(&self.snapshot.frame_samples);

            div()
                .id("dev_perf_monitor_expanded")
                .absolute()
                .left(current_pos.x)
                .top(current_pos.y)
                .w(px(EXPANDED_WIDTH))
                .rounded(px(10.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_mouse_up(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cx.stop_propagation();
                            cb(window, cx);
                        }
                    }
                })
                .on_mouse_up_out(MouseButton::Left, {
                    let cb = end_drag_action.clone();
                    move |_event, window, cx| {
                        if is_dragging {
                            cb(window, cx);
                        }
                    }
                })
                .on_click(|_, _, cx| cx.stop_propagation())
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                // Header (Draggable)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_move()
                        .on_mouse_down(MouseButton::Left, {
                            let end_cb = end_drag_action.clone();
                            move |event, window, cx| {
                                cx.stop_propagation();
                                if is_dragging {
                                    end_cb(window, cx);
                                } else {
                                    on_start_drag(event.position, pos_clone, window, cx);
                                }
                            }
                        })
                        .on_mouse_move(move |event, window, cx| {
                            if event.dragging() {
                                cx.stop_propagation();
                                on_drag_move(event.position, true, window, cx);
                            } else if is_dragging {
                                cx.stop_propagation();
                                on_drag_move(event.position, false, window, cx);
                            }
                        })
                        .on_mouse_up(MouseButton::Left, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                cx.stop_propagation();
                                cb(window, cx);
                            }
                        })
                        .on_mouse_up_out(MouseButton::Left, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                if is_dragging {
                                    cb(window, cx);
                                }
                            }
                        })
                        .on_mouse_up(MouseButton::Right, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                cx.stop_propagation();
                                cb(window, cx);
                            }
                        })
                        .on_mouse_up_out(MouseButton::Right, {
                            let cb = end_drag_action.clone();
                            move |_event, window, cx| {
                                if is_dragging {
                                    cb(window, cx);
                                }
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().size(px(8.0)).rounded_full().bg(status_color))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.text_primary)
                                        .child("Dev Profiler HUD"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .px(px(4.0))
                                        .py(px(1.0))
                                        .rounded(px(4.0))
                                        .bg(theme.input_bg)
                                        .border_1()
                                        .border_color(theme.card_border)
                                        .text_color(theme.text_muted)
                                        .child("DEV ONLY"),
                                ),
                        )
                        .child({
                            let is_min_hovered = self.snapshot.hovered_control == Some("min");
                            let min_spring = hover_spring(if is_min_hovered { 0.5 } else { 0.0 });
                            let on_hover_min = on_hover_control.clone();

                            let is_close_hovered = self.snapshot.hovered_control == Some("close");
                            let close_spring =
                                hover_spring(if is_close_hovered { 0.6 } else { 0.0 });
                            let on_hover_close = on_hover_control.clone();

                            div()
                                .flex()
                                .items_center()
                                .gap(px(2.0))
                                .child(
                                    IconButton::new("dev_perf_minimize_btn", "icons/minus.svg")
                                        .button_size(px(24.0))
                                        .icon_size(px(14.0))
                                        .spring(min_spring, theme.accent_cyan)
                                        .on_hover(move |hov, window, cx| {
                                            if let Some(ref h) = on_hover_min {
                                                h("min", hov, window, cx);
                                            }
                                        })
                                        .on_mouse_down(move |window, cx| {
                                            on_toggle(window, cx);
                                        }),
                                )
                                .child(
                                    IconButton::new("dev_perf_close_btn", "icons/x.svg")
                                        .button_size(px(24.0))
                                        .icon_size(px(14.0))
                                        .destructive(true)
                                        .spring(close_spring, theme.accent_red)
                                        .on_hover(move |hov, window, cx| {
                                            if let Some(ref h) = on_hover_close {
                                                h("close", hov, window, cx);
                                            }
                                        })
                                        .on_mouse_down(move |window, cx| {
                                            on_close(window, cx);
                                        }),
                                )
                        }),
                )
                // Metrics Hero Block with Monospace Right-Aligned Digits
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .p(px(8.0))
                        .rounded(px(8.0))
                        .bg(theme.input_bg)
                        .border_1()
                        .border_color(theme.input_border)
                        // Top Hero Row: Big FPS + Target status
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .items_baseline()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .font_family("Consolas")
                                                .text_2xl()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(status_color)
                                                .child(if self.snapshot.displayed_fps <= 0.0 {
                                                    "IDLE".to_string()
                                                } else {
                                                    format!("{:.0}", self.snapshot.displayed_fps)
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme.text_muted)
                                                .child(if self.snapshot.displayed_fps <= 0.0 {
                                                    ""
                                                } else {
                                                    "FPS"
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_end()
                                        .font_family("Consolas")
                                        .child(
                                            div().text_xs().text_color(theme.text_primary).child(
                                                format!(
                                                    "TIME: {:.1} ms",
                                                    self.snapshot.displayed_frame_ms
                                                ),
                                            ),
                                        )
                                        .child(div().text_xs().text_color(theme.text_muted).child(
                                            format!(
                                                "P95:  {:.1} ms",
                                                self.snapshot.displayed_p95_ms
                                            ),
                                        )),
                                ),
                        )
                        // Secondary Stats Row: Drop %, CPU render time, and Memory
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .pt(px(4.0))
                                .border_t_1()
                                .border_color(theme.card_border)
                                .font_family("Consolas")
                                .text_xs()
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("DROP:"))
                                        .child(
                                            div()
                                                .text_color(
                                                    if self.snapshot.displayed_drop_rate > 0.0 {
                                                        theme.accent_red
                                                    } else {
                                                        theme.accent_green
                                                    },
                                                )
                                                .child(format!(
                                                    "{:.1}%",
                                                    self.snapshot.displayed_drop_rate
                                                )),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("CPU:"))
                                        .child(
                                            div().text_color(theme.text_primary).child(format!(
                                                "{:.1}ms",
                                                self.snapshot.cpu_draw_ms
                                            )),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(4.0))
                                        .child(div().text_color(theme.text_muted).child("MEM:"))
                                        .child(div().text_color(theme.text_primary).child(ram_str)),
                                ),
                        ),
                )
                // Frame Time Graph (Reusing shared stepped history graph from hardware pages)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Frame Time History")
                                .child("16.6ms target"),
                        )
                        .child(render_stepped_history_graph_sized(
                            &frame_samples,
                            None,
                            Instant::now(),
                            &theme,
                            HistoryGraphPalette::Semantic,
                            "dev_perf_chart_glide",
                            (33.3, 33.3),
                            "ms",
                            px(48.0),
                        )),
                )
                // FPS Bottlenecks Inspector & Controls
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .p(px(8.0))
                        .rounded(px(8.0))
                        .bg(theme.input_bg)
                        .border_1()
                        .border_color(theme.input_border)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.accent_cyan)
                                .child("FPS Bottlenecks Inspector"),
                        )
                        // Route tree complexity
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap(px(8.0))
                                .text_xs()
                                .child(
                                    div()
                                        .flex_1()
                                        .truncate()
                                        .text_color(theme.text_muted)
                                        .child(format!("Page: {}", self.current_route.title())),
                                )
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .px(px(5.0))
                                        .py(px(1.0))
                                        .rounded(px(4.0))
                                        .bg(theme.card_bg)
                                        .border_1()
                                        .border_color(theme.card_border)
                                        .text_color(theme.text_primary)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(route_weight),
                                ),
                        )
                        // Telemetry Poller row + Toggle
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .child("Telemetry (500ms):"),
                                )
                                .child({
                                    let is_tel_hovered =
                                        self.snapshot.hovered_control == Some("telemetry");
                                    let tel_spring =
                                        hover_spring(if is_tel_hovered { 0.5 } else { 0.0 });
                                    let on_hover_tel = on_hover_control.clone();
                                    let tel_accent = if self.snapshot.freeze_telemetry {
                                        theme.accent_red
                                    } else {
                                        theme.accent_green
                                    };

                                    Chip::new(
                                        "dev_perf_freeze_telemetry_btn",
                                        if self.snapshot.freeze_telemetry {
                                            "PAUSED"
                                        } else {
                                            "ACTIVE"
                                        },
                                    )
                                    .destructive(self.snapshot.freeze_telemetry)
                                    .selected(!self.snapshot.freeze_telemetry)
                                    .spring(tel_spring, tel_accent)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_tel {
                                            h("telemetry", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_freeze_telemetry(window, cx);
                                        },
                                    )
                                }),
                        )
                        // 60 FPS Chart Animation Loop row + Toggle
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .child("Chart Glide (16ms):"),
                                )
                                .child({
                                    let is_chart_hovered =
                                        self.snapshot.hovered_control == Some("chart_anim");
                                    let chart_spring =
                                        hover_spring(if is_chart_hovered { 0.5 } else { 0.0 });
                                    let on_hover_chart = on_hover_control.clone();
                                    let chart_accent = if self.snapshot.disable_chart_animation {
                                        theme.accent_red
                                    } else {
                                        theme.accent_green
                                    };

                                    Chip::new(
                                        "dev_perf_chart_anim_btn",
                                        if self.snapshot.disable_chart_animation {
                                            "OFF"
                                        } else {
                                            "ON"
                                        },
                                    )
                                    .destructive(self.snapshot.disable_chart_animation)
                                    .selected(!self.snapshot.disable_chart_animation)
                                    .spring(chart_spring, chart_accent)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_chart {
                                            h("chart_anim", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_chart_anim(window, cx);
                                        },
                                    )
                                }),
                        )
                        // Continuous Drive Mode row + Toggle (from longbridge/gpui-component/fps)
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(div().text_color(theme.text_muted).child("Continuous Mode:"))
                                .child({
                                    let is_cont_hovered =
                                        self.snapshot.hovered_control == Some("continuous");
                                    let cont_spring =
                                        hover_spring(if is_cont_hovered { 0.5 } else { 0.0 });
                                    let on_hover_cont = on_hover_control.clone();

                                    Chip::new(
                                        "dev_perf_continuous_btn",
                                        if self.snapshot.continuous_mode {
                                            "DRIVING"
                                        } else {
                                            "IDLE-AWARE"
                                        },
                                    )
                                    .selected(self.snapshot.continuous_mode)
                                    .spring(cont_spring, theme.accent_blue)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_cont {
                                            h("continuous", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_continuous(window, cx);
                                        },
                                    )
                                }),
                        ),
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_monitor_initial_metrics() {
        let mut state = DevPerfMonitorState::new();
        assert!(state.enabled);
        assert!(state.minimized);
        assert_eq!(state.hovered_control, None);
        state.set_hovered_control(Some("expand"));
        assert_eq!(state.hovered_control, Some("expand"));
        assert_eq!(state.snapshot().hovered_control, Some("expand"));
        state.record_frame(2.0);
        assert!(state.displayed_fps > 0.0);
    }

    #[test]
    fn perf_monitor_vsync_tolerance() {
        // 58-60 fps on a 60Hz display is healthy
        let healthy = fps_status_color(58.5);
        assert_eq!(healthy, rgba(0x10B9_81FF));

        // 45 fps is warning
        let warning = fps_status_color(45.0);
        assert_eq!(warning, rgba(0xF59E_0BFF));

        // 20 fps is critical
        let critical = fps_status_color(20.0);
        assert_eq!(critical, rgba(0xEF44_44FF));

        // 0 fps is idle
        let idle = fps_status_color(0.0);
        assert_eq!(idle, rgba(0x8888_88FF));
    }

    #[test]
    fn perf_monitor_bounds_clamping() {
        let mut state = DevPerfMonitorState::new();
        state.position = Some(point(px(-100.0), px(-50.0)));
        let clamped = state.current_pos(px(800.0), px(600.0));
        assert!(clamped.x >= px(BOUNDS_PADDING));
        assert!(clamped.y >= px(TITLEBAR_HEIGHT));
    }
}
