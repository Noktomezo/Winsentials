use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{App, Pixels, Point, Rgba, Window, point, px, rgba};

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
pub(crate) fn fps_status_color(fps: f32) -> Rgba {
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
