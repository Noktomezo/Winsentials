use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{Context, FocusHandle, SharedString, Window};

use crate::entities::cleanup::CleanupState;
use crate::entities::{AppConfig, TelemetryData, load_config};
use crate::features::discord_rpc::{DiscordRpcActivity, DiscordRpcManager};
use crate::features::navigation::AppRoute;
use crate::features::tray::TrayManager;
use crate::shared::ui::TooltipState;

mod actions_cleanup_startup;
mod actions_dropdown;
mod actions_nav;
mod actions_settings;
mod actions_updater;
mod render;
mod render_hud;
mod render_panel;

#[cfg(test)]
mod tests;

#[allow(clippy::struct_excessive_bools)]
pub struct AppView {
    pub(crate) sidebar_expanded: bool,
    pub(crate) sidebar_toggle_hovered: bool,
    pub(crate) hovered_win_control: Option<&'static str>,
    pub(crate) hovered_titlebar_breadcrumb: Option<&'static str>,
    pub(crate) current_route: AppRoute,
    pub(crate) hovered_route: Option<AppRoute>,
    pub(crate) history_back: Vec<AppRoute>,
    pub(crate) history_forward: Vec<AppRoute>,
    pub(crate) focus_handle: Option<FocusHandle>,
    pub(crate) current_locale: &'static str,
    pub(crate) config: AppConfig,
    pub(crate) discord_rpc_manager: Arc<Mutex<DiscordRpcManager>>,
    pub(crate) _tray_manager: Option<TrayManager>,
    pub(crate) open_item_id: String,
    pub(crate) quit_item_id: String,
    pub(crate) open_dropdown: Option<&'static str>,
    pub(crate) opening_dropdown: Option<&'static str>,
    pub(crate) open_dropdown_upward: bool,
    pub(crate) closing_dropdown: Option<&'static str>,
    pub(crate) hovered_dropdown: Option<&'static str>,
    pub(crate) hovered_option: Option<(&'static str, &'static str)>,
    pub(crate) pending_selection: Option<(&'static str, &'static str)>,
    pub(crate) gpu_engine_slots: HashMap<(usize, usize), &'static str>,
    pub(crate) hovered_telemetry_card: Option<SharedString>,
    pub(crate) windows_build: u32,
    pub(crate) active_tooltip: Option<TooltipState>,
    pub(crate) telemetry: TelemetryData,
    pub(crate) toasts: Vec<crate::shared::ui::ToastData>,
    pub(crate) closing_toast_id: Option<SharedString>,
    pub(crate) hovered_toast_button: Option<(SharedString, usize)>,
    pub(crate) toast_stack_expanded: bool,
    pub(crate) startup_entries: Vec<crate::entities::startup::StartupEntry>,
    pub(crate) startup_filter: Option<crate::entities::startup::StartupSource>,
    pub(crate) startup_search_query: String,
    pub(crate) startup_search_focused: bool,
    pub(crate) startup_search_hovered: bool,
    pub(crate) startup_search_selection: Option<(usize, usize)>,
    pub(crate) startup_search_focus: Option<gpui::FocusHandle>,
    pub(crate) startup_open_menu_id: Option<String>,
    pub(crate) hovered_startup_card: Option<String>,
    pub(crate) cleanup: CleanupState,
    pub(crate) update_state: crate::features::updater::UpdateState,
    pub(crate) http_client: reqwest::Client,
    #[cfg(debug_assertions)]
    pub dev_perf_monitor: crate::widgets::dev_perf_monitor::DevPerfMonitorState,
}

impl AppView {
    #[must_use]
    pub fn new() -> Self {
        let sys_info = crate::entities::SystemInfo::fetch();
        let windows_build = sys_info
            .build_number
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(22000);

        let mut config = load_config();

        let mut discord_manager = DiscordRpcManager::new(windows_build);
        if config.discord_rpc != DiscordRpcActivity::Disabled {
            discord_manager.set_activity(config.discord_rpc);
        }
        let discord_rpc_manager = Arc::new(Mutex::new(discord_manager));

        let tray_manager = TrayManager::new();
        let open_item_id = tray_manager.open_item_id.clone();
        let quit_item_id = tray_manager.quit_item_id.clone();
        let startup_entries = crate::entities::startup::fetch_all_startup_entries();

        if config.snapkey != crate::entities::tweaks::input::SnapKeyPreset::Off {
            if let Err(error) = crate::entities::tweaks::input::set_snapkey_preset(config.snapkey) {
                eprintln!("failed to restore SnapKey preset: {error}");
                config.snapkey = crate::entities::tweaks::input::SnapKeyPreset::Off;
            }
        }

        Self {
            sidebar_expanded: false,
            sidebar_toggle_hovered: false,
            hovered_win_control: None,
            hovered_titlebar_breadcrumb: None,
            current_route: AppRoute::Dashboard,
            hovered_route: None,
            history_back: Vec::new(),
            history_forward: Vec::new(),
            focus_handle: None,
            current_locale: "system",
            config,
            discord_rpc_manager,
            _tray_manager: Some(tray_manager),
            open_item_id,
            quit_item_id,
            open_dropdown: None,
            opening_dropdown: None,
            open_dropdown_upward: false,
            closing_dropdown: None,
            hovered_dropdown: None,
            hovered_option: None,
            pending_selection: None,
            gpu_engine_slots: HashMap::new(),
            hovered_telemetry_card: None,
            windows_build,
            active_tooltip: None,
            telemetry: TelemetryData::fetch(),
            toasts: Vec::new(),
            closing_toast_id: None,
            hovered_toast_button: None,
            toast_stack_expanded: false,
            startup_entries,
            startup_filter: None,
            startup_search_query: String::new(),
            startup_search_focused: false,
            startup_search_hovered: false,
            startup_search_selection: None,
            startup_search_focus: None,
            startup_open_menu_id: None,
            hovered_startup_card: None,
            cleanup: CleanupState::default(),
            update_state: crate::features::updater::UpdateState::Idle,
            http_client: reqwest::Client::builder()
                .user_agent(concat!("Winsentials/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            #[cfg(debug_assertions)]
            dev_perf_monitor: crate::widgets::dev_perf_monitor::DevPerfMonitorState::new(),
        }
    }

    #[allow(clippy::unused_self)]
    pub fn start_telemetry_polling(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;

                let next_data = cx
                    .background_executor()
                    .spawn(async { TelemetryData::fetch() })
                    .await;

                let updated = this.update(cx, |this, cx| {
                    #[cfg(debug_assertions)]
                    if this.dev_perf_monitor.freeze_telemetry {
                        return;
                    }
                    this.telemetry = next_data;
                    cx.notify();
                });

                if updated.is_err() {
                    break;
                }
            }
        })
        .detach();

        // 60 FPS smooth continuous chart gliding when viewing telemetry detail pages
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;

                let updated = this.update(cx, |this, cx| {
                    #[cfg(debug_assertions)]
                    if this.dev_perf_monitor.disable_chart_animation {
                        return;
                    }
                    if matches!(
                        this.current_route,
                        AppRoute::CpuDetail
                            | AppRoute::RamDetail
                            | AppRoute::DiskDetail(_)
                            | AppRoute::NetworkDetail(_)
                            | AppRoute::GpuDetail(_)
                    ) {
                        cx.notify();
                    }
                });

                if updated.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    #[must_use]
    pub fn minimize_to_tray(&self) -> bool {
        self.config.minimize_to_tray
    }

    pub fn handle_window_close(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.config.minimize_to_tray {
            crate::features::tray::hide_main_window();
        } else {
            cx.quit();
        }
    }

    pub fn start_tray_listener(&mut self, cx: &mut Context<Self>) {
        let open_id = self.open_item_id.clone();
        let quit_id = self.quit_item_id.clone();
        cx.spawn(async move |_this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;

                while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                    if matches!(
                        event,
                        tray_icon::TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            button_state: tray_icon::MouseButtonState::Up,
                            ..
                        } | tray_icon::TrayIconEvent::DoubleClick {
                            button: tray_icon::MouseButton::Left,
                            ..
                        }
                    ) {
                        crate::features::tray::show_main_window();
                    }
                }

                while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    if event.id.0 == open_id {
                        crate::features::tray::show_main_window();
                    } else if event.id.0 == quit_id {
                        cx.update(|cx| {
                            cx.quit();
                        });
                    }
                }
            }
        })
        .detach();
    }
}

impl Default for AppView {
    fn default() -> Self {
        Self::new()
    }
}