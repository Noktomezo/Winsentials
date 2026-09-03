use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px};

#[cfg(debug_assertions)]
use gpui::{InteractiveElement, MouseButton, Pixels, Point};

use crate::entities::cleanup::{CleanupCategory, CleanupState};
use crate::entities::{AppConfig, TelemetryData, load_config, save_config};
use crate::features::discord_rpc::{DiscordRpcActivity, DiscordRpcManager};
use crate::features::navigation::AppRoute;
use crate::features::tray::TrayManager;
use crate::pages::{CleanupPage, render_route};
use crate::shared::theme::{Theme, ThemeMode, ThemePalette};
use crate::shared::ui::{Tooltip, TooltipState};
use crate::widgets::sidebar::Sidebar;
use crate::widgets::titlebar::Titlebar;

#[allow(clippy::struct_excessive_bools)]
pub struct AppView {
    sidebar_expanded: bool,
    sidebar_toggle_hovered: bool,
    hovered_win_control: Option<&'static str>,
    hovered_titlebar_breadcrumb: Option<&'static str>,
    current_route: AppRoute,
    hovered_route: Option<AppRoute>,
    current_locale: &'static str,
    config: AppConfig,
    discord_rpc_manager: Arc<Mutex<DiscordRpcManager>>,
    _tray_manager: Option<TrayManager>,
    open_item_id: String,
    quit_item_id: String,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    gpu_engine_slots: HashMap<(usize, usize), &'static str>,
    hovered_telemetry_card: Option<SharedString>,
    windows_build: u32,
    active_tooltip: Option<TooltipState>,
    telemetry: TelemetryData,
    toasts: Vec<crate::shared::ui::ToastData>,
    closing_toast_id: Option<SharedString>,
    hovered_toast_button: Option<(SharedString, usize)>,
    toast_stack_expanded: bool,
    startup_entries: Vec<crate::entities::startup::StartupEntry>,
    startup_filter: Option<crate::entities::startup::StartupSource>,
    startup_search_query: String,
    startup_search_focused: bool,
    startup_search_hovered: bool,
    startup_search_selection: Option<(usize, usize)>,
    startup_search_focus: Option<gpui::FocusHandle>,
    startup_open_menu_id: Option<String>,
    hovered_startup_card: Option<String>,
    cleanup: CleanupState,
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
            current_locale: "system",
            config,
            discord_rpc_manager,
            _tray_manager: Some(tray_manager),
            open_item_id,
            quit_item_id,
            open_dropdown: None,
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

    pub fn toggle_sidebar(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_expanded = !self.sidebar_expanded;
        self.active_tooltip = None;
        cx.notify();
    }

    pub fn navigate_to(&mut self, route: AppRoute, _window: &mut Window, cx: &mut Context<Self>) {
        if self.current_route != route {
            self.current_route = route;
            if let Ok(mut mgr) = self.discord_rpc_manager.lock() {
                mgr.set_route(route);
            }
            self.open_dropdown = None;
            self.closing_dropdown = None;
            self.hovered_dropdown = None;
            self.hovered_option = None;
            self.pending_selection = None;
            self.hovered_telemetry_card = None;
            self.hovered_titlebar_breadcrumb = None;
            self.active_tooltip = None;
            if route == AppRoute::Startup {
                self.startup_entries = crate::entities::startup::fetch_all_startup_entries();
            }
            if route == AppRoute::Cleanup && !self.cleanup.scanned_once {
                self.refresh_cleanup(cx);
            }
            cx.notify();
        }
    }

    fn refresh_cleanup(&mut self, cx: &mut Context<Self>) {
        if self.cleanup.scanning || self.cleanup.cleaning {
            return;
        }
        self.cleanup.scanning = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let files = cx
                .background_executor()
                .spawn(async { crate::entities::cleanup::scan_cleanup_targets() });
            let devices = cx
                .background_executor()
                .spawn(async { crate::entities::cleanup::scan_unused_devices() });
            let mut snapshot = files.await;
            snapshot.targets.extend(devices.await);
            if let Err(error) = this.update(cx, |this, cx| {
                this.cleanup.apply_snapshot(snapshot);
                cx.notify();
            }) {
                eprintln!("cleanup scan update failed: {error}");
            }
        })
        .detach();
    }

    fn clean_cleanup(&mut self, category: Option<CleanupCategory>, cx: &mut Context<Self>) {
        if self.cleanup.scanning || self.cleanup.cleaning {
            return;
        }
        let selected = self
            .cleanup
            .snapshot
            .targets
            .iter()
            .filter(|target| {
                self.cleanup.selected.contains(&target.id)
                    && category.is_none_or(|value| value == target.category)
            })
            .map(|target| target.id.clone())
            .collect::<std::collections::HashSet<_>>();
        if selected.is_empty() {
            return;
        }
        let confirmed = rfd::MessageDialog::new()
            .set_title(rust_i18n::t!("cleanup.confirm_title").as_ref())
            .set_description(rust_i18n::t!("cleanup.confirm_body").as_ref())
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes;
        if !confirmed {
            return;
        }

        let snapshot = self.cleanup.snapshot.clone();
        self.cleanup.cleaning = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let report = cx
                .background_executor()
                .spawn(
                    async move { crate::entities::cleanup::clean_selected(&snapshot, &selected) },
                )
                .await;
            let files = cx
                .background_executor()
                .spawn(async { crate::entities::cleanup::scan_cleanup_targets() });
            let devices = cx
                .background_executor()
                .spawn(async { crate::entities::cleanup::scan_unused_devices() });
            let mut refreshed = files.await;
            refreshed.targets.extend(devices.await);
            if let Err(error) = this.update(cx, |this, cx| {
                this.cleanup.cleaning = false;
                this.cleanup.selected.clear();
                this.cleanup.apply_snapshot(refreshed);
                let size = crate::entities::cleanup::format_bytes(report.removed_bytes);
                let title = if report.failures == 0 {
                    rust_i18n::t!("cleanup.done", size = size).to_string()
                } else {
                    rust_i18n::t!(
                        "cleanup.done_with_errors",
                        size = size,
                        count = report.failures
                    )
                    .to_string()
                };
                this.show_toast(
                    crate::shared::ui::ToastData::new("cleanup_result", title)
                        .icon("icons/broom.svg"),
                    cx,
                );
            }) {
                eprintln!("cleanup result update failed: {error}");
            }
        })
        .detach();
    }

    pub fn set_hovered_route(
        &mut self,
        route: AppRoute,
        is_hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_route != Some(route) {
                self.hovered_route = Some(route);
                cx.notify();
            }
        } else if self.hovered_route == Some(route) {
            self.hovered_route = None;
            cx.notify();
        }
    }

    pub fn set_hovered_titlebar_breadcrumb(
        &mut self,
        id: &'static str,
        is_hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_titlebar_breadcrumb != Some(id) {
                self.hovered_titlebar_breadcrumb = Some(id);
                cx.notify();
            }
        } else if self.hovered_titlebar_breadcrumb == Some(id) {
            self.hovered_titlebar_breadcrumb = None;
            cx.notify();
        }
    }

    pub fn set_active_tooltip(&mut self, tooltip: Option<TooltipState>, cx: &mut Context<Self>) {
        if self.active_tooltip != tooltip {
            self.active_tooltip = tooltip;
            cx.notify();
        }
    }

    pub fn show_toast(&mut self, mut toast: crate::shared::ui::ToastData, cx: &mut Context<Self>) {
        let duration = toast.duration;
        let toast_id = toast.id.clone();

        if let Some(existing) = self.toasts.iter_mut().find(|t| t.id == toast_id) {
            existing.count += 1;
            existing.title = toast.title;
            existing.description = toast.description;
            existing.variant = toast.variant;
            existing.buttons = toast.buttons;
            existing.progress = toast.progress;
            existing.duration = toast.duration;
        } else {
            toast.count = 1;
            self.toasts.push(toast);
        }
        cx.notify();

        if let Some(dur) = duration {
            let t_id = toast_id.to_string();
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(dur).await;
                let _ = this.update(cx, |this, cx| {
                    this.dismiss_toast(&t_id, cx);
                });
            })
            .detach();
        }
    }

    pub fn dismiss_toast(&mut self, toast_id: &str, cx: &mut Context<Self>) {
        if self.toasts.iter().any(|t| t.id == toast_id)
            && self.closing_toast_id.as_deref() != Some(toast_id)
        {
            let t_id_shared: SharedString = toast_id.to_string().into();
            self.closing_toast_id = Some(t_id_shared.clone());
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(160))
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.toasts.retain(|t| t.id != t_id_shared);
                    if this.closing_toast_id == Some(t_id_shared) {
                        this.closing_toast_id = None;
                    }
                    cx.notify();
                });
            })
            .detach();
        }
    }

    pub fn set_hovered_toast_button(
        &mut self,
        toast_id: &str,
        index: usize,
        is_hovered: bool,
        cx: &mut Context<Self>,
    ) {
        let key = (toast_id.to_string().into(), index);
        if is_hovered {
            if self.hovered_toast_button.as_ref() != Some(&key) {
                self.hovered_toast_button = Some(key);
                cx.notify();
            }
        } else if self.hovered_toast_button.as_ref() == Some(&key) {
            self.hovered_toast_button = None;
            cx.notify();
        }
    }

    pub fn set_toast_stack_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        if self.toast_stack_expanded != expanded {
            self.toast_stack_expanded = expanded;
            cx.notify();
        }
    }

    pub fn refresh_startup_entries(&mut self, cx: &mut Context<Self>) {
        self.startup_entries = crate::entities::startup::fetch_all_startup_entries();
        cx.notify();
    }

    pub fn toggle_startup(
        &mut self,
        entry: &crate::entities::startup::StartupEntry,
        cx: &mut Context<Self>,
    ) {
        crate::entities::startup::toggle_startup_entry(entry);
        self.refresh_startup_entries(cx);
    }

    pub fn delete_startup(
        &mut self,
        entry: &crate::entities::startup::StartupEntry,
        cx: &mut Context<Self>,
    ) {
        crate::entities::startup::delete_startup_entry(entry);
        self.refresh_startup_entries(cx);
    }

    pub fn set_startup_filter(
        &mut self,
        filter: Option<crate::entities::startup::StartupSource>,
        cx: &mut Context<Self>,
    ) {
        self.startup_filter = filter;
        self.startup_search_focused = false;
        cx.notify();
    }

    pub fn set_startup_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.startup_search_query = query;
        cx.notify();
    }

    pub fn set_startup_search_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        self.startup_search_hovered = hovered;
        cx.notify();
    }

    pub fn set_startup_search_focused(&mut self, focused: bool, cx: &mut Context<Self>) {
        self.startup_search_focused = focused;
        if !focused {
            self.startup_search_selection = None;
        }
        cx.notify();
    }

    pub fn set_startup_search_selection(
        &mut self,
        selection: Option<(usize, usize)>,
        cx: &mut Context<Self>,
    ) {
        self.startup_search_selection = selection;
        cx.notify();
    }

    pub fn set_startup_menu(&mut self, menu_id: Option<String>, cx: &mut Context<Self>) {
        self.startup_open_menu_id = menu_id;
        if self.startup_open_menu_id.is_some() {
            self.startup_search_focused = false;
            self.startup_search_selection = None;
        }
        cx.notify();
    }

    pub fn show_explorer_restart_toast(&mut self, cx: &mut Context<Self>) {
        let restart_toast = crate::shared::ui::ToastData::new(
            "explorer_restart",
            rust_i18n::t!("tweaks.restart_explorer_title"),
        )
        .description(rust_i18n::t!("tweaks.restart_explorer_desc"))
        .icon("icons/refresh-cw.svg")
        .duration(None)
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.restart_now"))
                .variant(crate::shared::ui::ToastButtonVariant::Primary)
                .on_click(|_window, _cx| {
                    crate::entities::tweaks::context_menu::classic_menu::restart_explorer();
                }),
        )
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.later"))
                .variant(crate::shared::ui::ToastButtonVariant::Secondary),
        );
        self.show_toast(restart_toast, cx);
    }

    pub fn show_logoff_toast(&mut self, cx: &mut Context<Self>) {
        let logoff_toast = crate::shared::ui::ToastData::new(
            "system_logoff",
            rust_i18n::t!("tweaks.logoff_title"),
        )
        .description(rust_i18n::t!("tweaks.logoff_desc"))
        .icon("icons/log-out.svg")
        .duration(None)
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.logoff_now"))
                .variant(crate::shared::ui::ToastButtonVariant::Primary)
                .on_click(|_window, _cx| {
                    let _ = system_shutdown::logout();
                }),
        )
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.later"))
                .variant(crate::shared::ui::ToastButtonVariant::Secondary),
        );
        self.show_toast(logoff_toast, cx);
    }

    pub fn show_reboot_toast(&mut self, cx: &mut Context<Self>) {
        let reboot_toast = crate::shared::ui::ToastData::new(
            "system_reboot",
            rust_i18n::t!("tweaks.reboot_title"),
        )
        .description(rust_i18n::t!("tweaks.reboot_desc"))
        .icon("icons/power.svg")
        .duration(None)
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.reboot_now"))
                .variant(crate::shared::ui::ToastButtonVariant::Primary)
                .on_click(|_window, _cx| {
                    let _ = system_shutdown::reboot();
                }),
        )
        .button(
            crate::shared::ui::ToastButton::new(rust_i18n::t!("tweaks.later"))
                .variant(crate::shared::ui::ToastButtonVariant::Secondary),
        );
        self.show_toast(reboot_toast, cx);
    }

    pub fn set_hovered_telemetry_card(
        &mut self,
        card_id: SharedString,
        is_hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_telemetry_card.as_ref() != Some(&card_id) {
                self.hovered_telemetry_card = Some(card_id);
                cx.notify();
            }
        } else if self.hovered_telemetry_card.as_ref() == Some(&card_id) {
            self.hovered_telemetry_card = None;
            cx.notify();
        }
    }

    pub fn set_hovered_startup_card(&mut self, card_id: Option<String>, cx: &mut Context<Self>) {
        if self.hovered_startup_card != card_id {
            self.hovered_startup_card = card_id;
            cx.notify();
        }
    }

    pub fn set_language(&mut self, lang: &str, cx: &mut Context<Self>) {
        let (code, state_key): (&'static str, &'static str) = match lang {
            "en" => ("en", "en"),
            "ru" => ("ru", "ru"),
            _ => ("ru", "system"),
        };
        rust_i18n::set_locale(code);
        self.current_locale = state_key;
        cx.notify();
    }

    #[allow(clippy::unused_self)]
    pub fn set_palette(&mut self, palette: &str, cx: &mut Context<Self>) {
        let current_theme = Theme::get(cx);
        let pal_enum = match palette {
            "flexoki" => ThemePalette::Flexoki,
            _ => ThemePalette::Arclate,
        };
        let new_theme = Theme::build(pal_enum, current_theme.mode, current_theme.transparency);
        cx.set_global(new_theme);
        cx.notify();
    }

    #[allow(clippy::unused_self)]
    pub fn set_theme_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        let current_theme = Theme::get(cx);
        let mode_enum = match mode {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        };
        let new_theme = Theme::build(current_theme.palette, mode_enum, current_theme.transparency);
        cx.set_global(new_theme);
        cx.notify();
    }

    #[allow(clippy::unused_self)]
    pub fn set_transparency(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let current_theme = Theme::get(cx);
        let updated_theme = current_theme.with_transparency(enabled);
        cx.set_global(updated_theme);
        cx.notify();
    }

    pub fn toggle_tweak(&mut self, tweak_id: &'static str, enabled: bool, cx: &mut Context<Self>) {
        let all_tweaks = crate::entities::tweaks::get_all_tweaks();
        if let Some(tweak) = all_tweaks.iter().find(|t| t.id == tweak_id) {
            let set_applied = tweak.set_applied;
            let restart = tweak.restart;
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move { set_applied(enabled) })
                    .await;
                if let Err(update_error) = this.update(cx, move |this, cx| match result {
                    Ok(()) => {
                        crate::entities::SystemInfo::invalidate_cache();
                        crate::shared::shell_notify::notify_shell_change();
                        if let Ok(mut mgr) = this.discord_rpc_manager.lock() {
                            mgr.refresh_presence();
                        }
                        match restart {
                            crate::entities::tweaks::RestartRequirement::Explorer => {
                                this.show_explorer_restart_toast(cx);
                            }
                            crate::entities::tweaks::RestartRequirement::Logoff => {
                                this.show_logoff_toast(cx);
                            }
                            crate::entities::tweaks::RestartRequirement::Reboot => {
                                this.show_reboot_toast(cx);
                            }
                            crate::entities::tweaks::RestartRequirement::None => {}
                        }
                        cx.notify();
                    }
                    Err(error) => this.show_setting_error(tweak_id, &error, cx),
                }) {
                    eprintln!(
                        "failed to update tweak state after applying {tweak_id}: {update_error}"
                    );
                }
            })
            .detach();
        }
    }

    fn show_setting_error(&mut self, setting: &str, error: &str, cx: &mut Context<Self>) {
        eprintln!("failed to apply {setting}: {error}");
        let toast = crate::shared::ui::ToastData::new(
            "setting_apply_error",
            rust_i18n::t!("tweaks.apply_failed_title"),
        )
        .description(rust_i18n::t!("tweaks.apply_failed_desc"))
        .variant(crate::shared::ui::ToastVariant::Error);
        self.show_toast(toast, cx);
    }

    pub fn start_closing_dropdown(&mut self, name: &'static str, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(name) {
            self.open_dropdown = None;
            self.closing_dropdown = Some(name);
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(140))
                    .await;
                this.update(cx, |this, cx| {
                    if this.closing_dropdown == Some(name) {
                        this.closing_dropdown = None;
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn select_option(
        &mut self,
        dropdown: &'static str,
        val: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let static_val: &'static str = match val {
            "dark" => "dark",
            "light" => "light",
            "ru" => "ru",
            "en" => "en",
            "disabled" => "disabled",
            "enabled" => "enabled",
            "system" => "system",
            "notepad" => "notepad",
            "arclate" => "arclate",
            "flexoki" => "flexoki",
            "standard" => "standard",
            "balanced" => "balanced",
            "fast" => "fast",
            "ultra" => "ultra",
            "mild" => "mild",
            "aggressive" => "aggressive",
            "off" => "off",
            "wasd" => "wasd",
            "arrow_keys" => "arrow_keys",
            "esdf" => "esdf",
            "azerty" => "azerty",
            "playing" => "playing",
            "listening" => "listening",
            "watching" => "watching",
            "competing" => "competing",
            other => {
                self.show_setting_error(dropdown, &format!("unknown option: {other}"), cx);
                return;
            }
        };

        // Instantly transition the clicked item into the selected state (100% blue + checkmark)
        self.pending_selection = Some((dropdown, static_val));
        cx.notify();

        // After a brief tactile confirmation delay (100ms), apply setting and initiate smooth exit animation
        let dropdown_copy = dropdown;
        let val_copy = static_val;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            let apply_result = match dropdown_copy {
                "keyboard_repeat" => {
                    match crate::entities::tweaks::input::KeyboardRepeatPreset::from_id(val_copy) {
                        Some(preset) => {
                            cx.background_executor()
                                .spawn(async move {
                                    crate::entities::tweaks::input::set_keyboard_repeat_preset(
                                        preset,
                                    )
                                })
                                .await
                        }
                        None => Err(format!("unknown keyboard repeat preset: {val_copy}")),
                    }
                }
                "ctf_optimization" => {
                    match crate::entities::tweaks::input::CtfOptimizationPreset::from_id(val_copy) {
                        Some(preset) => {
                            cx.background_executor()
                                .spawn(async move {
                                    crate::entities::tweaks::input::set_ctf_preset(preset)
                                })
                                .await
                        }
                        None => Err(format!("unknown CTF preset: {val_copy}")),
                    }
                }
                "snapkey" => {
                    match crate::entities::tweaks::input::SnapKeyPreset::from_id(val_copy) {
                        Some(preset) => {
                            cx.background_executor()
                                .spawn(async move {
                                    crate::entities::tweaks::input::set_snapkey_preset(preset)
                                })
                                .await
                        }
                        None => Err(format!("unknown SnapKey preset: {val_copy}")),
                    }
                }
                _ => Ok(()),
            };
            this.update(cx, move |this, cx| {
                if let Err(error) = apply_result {
                    this.show_setting_error(dropdown_copy, &error, cx);
                } else if dropdown_copy == "palette" {
                    this.set_palette(val_copy, cx);
                } else if dropdown_copy == "theme" {
                    this.set_theme_mode(val_copy, cx);
                } else if dropdown_copy == "language" {
                    this.set_language(val_copy, cx);
                } else if dropdown_copy == "transparency" {
                    this.set_transparency(val_copy == "enabled", cx);
                } else if dropdown_copy == "snapkey" {
                    if let Some(preset) =
                        crate::entities::tweaks::input::SnapKeyPreset::from_id(val_copy)
                    {
                        this.save_snapkey_preset(preset, cx);
                    }
                }
                this.pending_selection = None;
                this.start_closing_dropdown(dropdown_copy, cx);
            })
            .ok();
        })
        .detach();
    }

    fn save_snapkey_preset(
        &mut self,
        preset: crate::entities::tweaks::input::SnapKeyPreset,
        cx: &mut Context<Self>,
    ) {
        self.config.snapkey = preset;
        if let Err(error) = crate::entities::config::save_config(&self.config) {
            self.show_setting_error("snapkey_config", &error, cx);
        }

        if preset != crate::entities::tweaks::input::SnapKeyPreset::Off
            && !self.config.minimize_to_tray
        {
            let on_enable_tray = cx.listener(|this, _event: &(), _window, cx| {
                this.toggle_minimize_to_tray(true, cx);
            });
            let enable_btn = crate::shared::ui::ToastButton::new(rust_i18n::t!(
                "tweaks.snapkey_tray_toast_action"
            ))
            .variant(crate::shared::ui::ToastButtonVariant::Primary)
            .icon("icons/check.svg")
            .on_click(move |window, cx| {
                on_enable_tray(&(), window, cx);
            });

            let toast = crate::shared::ui::ToastData::new(
                "snapkey_tray_prompt",
                rust_i18n::t!("tweaks.snapkey_tray_toast_title"),
            )
            .description(rust_i18n::t!("tweaks.snapkey_tray_toast_desc"))
            .variant(crate::shared::ui::ToastVariant::Info)
            .duration(Some(Duration::from_secs(8)))
            .button(enable_btn);

            self.show_toast(toast, cx);
        }
        cx.notify();
    }

    fn dropdown_required_space_below(name: &str) -> gpui::Pixels {
        let item_count: f32 = match name {
            "language" | "transparency" => 2.0,
            "theme" | "ctf_optimization" => 3.0,
            "snapkey" => 5.0,
            "palette" => 6.0,
            _ => 4.0,
        };
        // Item height (32px) + borders (2px) + vertical offset & breathing room (25px)
        px(item_count * 32.0 + 25.0)
    }

    pub fn toggle_dropdown(
        &mut self,
        name: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_dropdown == Some(name) {
            self.start_closing_dropdown(name, cx);
        } else if self.closing_dropdown == Some(name) {
            // Dropdown is currently playing its closing animation; ignore click so it doesn't immediately reopen
        } else {
            let mouse_y = window.mouse_position().y;
            let viewport_h = window.viewport_size().height;
            let space_below = viewport_h - mouse_y;
            let space_above = mouse_y - px(40.0);
            let required_space = Self::dropdown_required_space_below(name);
            self.open_dropdown_upward = space_below < required_space && space_above > space_below;
            self.open_dropdown = Some(name);
            self.closing_dropdown = None;
            cx.notify();
        }
    }

    pub fn set_hovered_dropdown(
        &mut self,
        name: &'static str,
        is_hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_dropdown != Some(name) {
                self.hovered_dropdown = Some(name);
                cx.notify();
            }
        } else if self.hovered_dropdown == Some(name) {
            self.hovered_dropdown = None;
            cx.notify();
        }
    }

    pub fn set_hovered_option(
        &mut self,
        dropdown: &'static str,
        option: &'static str,
        is_hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_hovered {
            if self.hovered_option != Some((dropdown, option)) {
                self.hovered_option = Some((dropdown, option));
                cx.notify();
            }
        } else if self.hovered_option == Some((dropdown, option)) {
            self.hovered_option = None;
            cx.notify();
        }
    }

    pub fn close_dropdowns(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(open) = self.open_dropdown {
            self.hovered_option = None;
            self.start_closing_dropdown(open, cx);
        }
    }

    pub fn set_gpu_engine_slot(
        &mut self,
        gpu_id: usize,
        slot_idx: usize,
        engine: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.gpu_engine_slots.insert((gpu_id, slot_idx), engine);
        self.open_dropdown = None;
        self.closing_dropdown = None;
        self.hovered_dropdown = None;
        self.hovered_option = None;
        cx.notify();
    }

    pub fn reset_gpu_engine_slots(&mut self, gpu_id: usize, cx: &mut Context<Self>) {
        for i in 0..4 {
            self.gpu_engine_slots.remove(&(gpu_id, i));
        }
        self.open_dropdown = None;
        self.closing_dropdown = None;
        self.hovered_dropdown = None;
        self.hovered_option = None;
        cx.notify();
    }

    pub fn toggle_minimize_to_tray(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.config.minimize_to_tray = enabled;
        let _ = save_config(&self.config);
        cx.notify();
    }

    pub fn toggle_autostart(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.config.autostart = enabled;
        let _ = crate::features::autostart::set_autostart(enabled, self.config.autostart_to_tray);
        let _ = save_config(&self.config);
        cx.notify();
    }

    pub fn toggle_autostart_to_tray(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.config.autostart_to_tray = enabled;
        let _ = crate::features::autostart::set_autostart(self.config.autostart, enabled);
        let _ = save_config(&self.config);
        cx.notify();
    }

    pub fn change_discord_rpc(
        &mut self,
        activity_str: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let activity = DiscordRpcActivity::from_str(activity_str);
        self.config.discord_rpc = activity;
        let _ = save_config(&self.config);
        if let Ok(mut mgr) = self.discord_rpc_manager.lock() {
            mgr.set_activity(activity);
        }
        let static_str = match activity {
            DiscordRpcActivity::Playing => "playing",
            DiscordRpcActivity::Listening => "listening",
            DiscordRpcActivity::Watching => "watching",
            DiscordRpcActivity::Competing => "competing",
            DiscordRpcActivity::Disabled => "disabled",
        };
        self.select_option("discord", static_str, window, cx);
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
                    .timer(Duration::from_millis(100))
                    .await;

                if let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                    if matches!(
                        event,
                        tray_icon::TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            ..
                        }
                    ) {
                        crate::features::tray::show_main_window();
                    }
                }

                if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
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

impl Render for AppView {
    #[allow(clippy::too_many_lines, unused_variables)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(debug_assertions)]
        let render_start = std::time::Instant::now();

        let theme = Theme::get(cx);
        let sidebar_expanded = self.sidebar_expanded;
        let sidebar_toggle_hovered = self.sidebar_toggle_hovered;
        let hovered_win_control = self.hovered_win_control;
        let hovered_titlebar_breadcrumb = self.hovered_titlebar_breadcrumb;
        let current_route = self.current_route;
        let hovered_route = self.hovered_route;
        let current_locale = self.current_locale;
        let open_dropdown = self.open_dropdown;
        let closing_dropdown = self.closing_dropdown;
        let hovered_dropdown = self.hovered_dropdown;
        let hovered_option = self.hovered_option;
        let pending_selection = self.pending_selection;
        let hovered_telemetry_card = self.hovered_telemetry_card.clone();
        let windows_build = self.windows_build;
        let active_tooltip = self.active_tooltip.clone();
        let telemetry = self.telemetry.clone();

        let on_hover_sidebar_toggle = cx.listener(|this, &hovered: &bool, _window, cx| {
            this.sidebar_toggle_hovered = hovered;
            cx.notify();
        });

        let on_toggle_sidebar = cx.listener(|this, _event: &(), window, cx| {
            this.toggle_sidebar(window, cx);
        });

        let on_navigate = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_hover_route = cx.listener(
            |this, &(route, is_hovered): &(AppRoute, bool), window, cx| {
                this.set_hovered_route(route, is_hovered, window, cx);
            },
        );

        let on_hover_win_control = cx.listener(
            |this, &(ctrl, is_hovered): &(&'static str, bool), _window, cx| {
                if is_hovered {
                    if this.hovered_win_control != Some(ctrl) {
                        this.hovered_win_control = Some(ctrl);
                        cx.notify();
                    }
                } else if this.hovered_win_control == Some(ctrl) {
                    this.hovered_win_control = None;
                    cx.notify();
                }
            },
        );

        let on_hover_titlebar_breadcrumb = cx.listener(
            |this, &(id, is_hovered): &(&'static str, bool), _window, cx| {
                this.set_hovered_titlebar_breadcrumb(id, is_hovered, cx);
            },
        );

        let titlebar_tooltip_listener =
            cx.listener(|this, tooltip: &Option<TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let sidebar_tooltip_listener =
            cx.listener(|this, tooltip: &Option<TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let page_tooltip_listener =
            cx.listener(|this, tooltip: &Option<TooltipState>, _window, cx| {
                this.set_active_tooltip(tooltip.clone(), cx);
            });

        let on_navigate_titlebar = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_close_win = cx.listener(|this, _event: &(), window, cx| {
            this.handle_window_close(window, cx);
        });

        let titlebar = Titlebar::new(
            current_route,
            sidebar_expanded,
            sidebar_toggle_hovered,
            hovered_win_control,
        )
        .hovered_breadcrumb(hovered_titlebar_breadcrumb)
        .on_hover_breadcrumb(move |id, is_hovered, window, cx| {
            on_hover_titlebar_breadcrumb(&(id, is_hovered), window, cx);
        })
        .on_navigate(move |route, window, cx| {
            on_navigate_titlebar(&route, window, cx);
        })
        .on_hover_sidebar_toggle(move |hovered, window, cx| {
            on_hover_sidebar_toggle(hovered, window, cx);
        })
        .on_toggle_sidebar(move |_event, window, cx| {
            on_toggle_sidebar(&(), window, cx);
        })
        .on_hover_win_control(move |ctrl, is_hovered, window, cx| {
            on_hover_win_control(&(ctrl, *is_hovered), window, cx);
        })
        .on_hover_tooltip(move |tooltip, window, cx| {
            titlebar_tooltip_listener(&tooltip, window, cx);
        })
        .on_close_window(move |window, cx| {
            on_close_win(&(), window, cx);
        });

        let sidebar = Sidebar::new(sidebar_expanded, current_route, hovered_route)
            .on_navigate(move |route, window, cx| {
                on_navigate(route, window, cx);
            })
            .on_hover_route(move |pair, window, cx| {
                on_hover_route(pair, window, cx);
            })
            .on_hover_tooltip(move |tooltip, window, cx| {
                sidebar_tooltip_listener(&tooltip, window, cx);
            });

        let on_hover_telemetry_card = cx.listener(
            |this, &(ref card_id, is_hovered): &(SharedString, bool), window, cx| {
                this.set_hovered_telemetry_card(card_id.clone(), is_hovered, window, cx);
            },
        );

        let on_toggle_tweak = cx.listener(
            |this, &(tweak_id, enabled): &(&'static str, bool), _window, cx| {
                this.toggle_tweak(tweak_id, enabled, cx);
            },
        );

        let on_change_keyboard_repeat = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("keyboard_repeat", preset, window, cx);
        });

        let on_change_ctf_optimization = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("ctf_optimization", preset, window, cx);
        });

        let on_change_snapkey = cx.listener(|this, preset: &str, window, cx| {
            this.select_option("snapkey", preset, window, cx);
        });

        let on_change_pal = cx.listener(|this, palette: &str, window, cx| {
            this.select_option("palette", palette, window, cx);
        });

        let on_change_lang = cx.listener(|this, lang: &str, window, cx| {
            this.select_option("language", lang, window, cx);
        });

        let on_change_th = cx.listener(|this, mode: &str, window, cx| {
            this.select_option("theme", mode, window, cx);
        });

        let on_change_trans = cx.listener(|this, enabled: &bool, _window, cx| {
            this.set_transparency(*enabled, cx);
        });

        let on_toggle_min_tray = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_minimize_to_tray(*enabled, cx);
        });

        let on_toggle_autostart = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_autostart(*enabled, cx);
        });

        let on_toggle_autostart_tray = cx.listener(|this, enabled: &bool, _window, cx| {
            this.toggle_autostart_to_tray(*enabled, cx);
        });

        let on_change_disc = cx.listener(|this, act: &str, window, cx| {
            this.change_discord_rpc(act, window, cx);
        });

        let on_select_gpu_engine = cx.listener(
            |this, &(gpu_id, slot_idx, engine): &(usize, usize, &'static str), _window, cx| {
                this.set_gpu_engine_slot(gpu_id, slot_idx, engine, cx);
            },
        );

        let on_reset_gpu_slots = cx.listener(|this, &gpu_id: &usize, _window, cx| {
            this.reset_gpu_engine_slots(gpu_id, cx);
        });

        let on_toggle_drop = cx.listener(|this, &name: &&'static str, window, cx| {
            this.toggle_dropdown(name, window, cx);
        });

        let on_hover_drop = cx.listener(
            |this, &(name, is_hovered): &(&'static str, bool), window, cx| {
                this.set_hovered_dropdown(name, is_hovered, window, cx);
            },
        );

        let on_hover_opt = cx.listener(
            |this,
             &(dropdown, opt, is_hovered): &(&'static str, &'static str, bool),
             window,
             cx| {
                this.set_hovered_option(dropdown, opt, is_hovered, window, cx);
            },
        );

        let on_close_drop = cx.listener(|this, _event: &(), window, cx| {
            this.close_dropdowns(window, cx);
        });

        // Main content area:
        // - Rounded ONLY on top-left by 8px to smooth junction corner between sidebar and titlebar
        // - All other corners are default (0px)
        // - Flush with window edges (no outer margins)
        let on_navigate_page = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });

        let on_toggle_startup = cx.listener(
            |this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                this.toggle_startup(entry, cx);
            },
        );

        let on_delete_startup = cx.listener(
            |this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                this.delete_startup(entry, cx);
            },
        );

        let on_open_startup_folder = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, _cx| {
                crate::entities::startup::open_startup_file_location(entry);
            },
        );

        let on_open_startup_source = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, _cx| {
                crate::entities::startup::open_startup_source_manager(entry);
            },
        );

        let on_copy_startup_path = cx.listener(
            |_this, entry: &crate::entities::startup::StartupEntry, _window, cx| {
                let path_to_copy = entry
                    .target_path
                    .as_deref()
                    .or(entry.command.as_deref())
                    .unwrap_or(&entry.raw_id);
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(path_to_copy.to_string()));
            },
        );

        let on_toggle_startup_menu = cx.listener(|this, menu_id: &Option<String>, _window, cx| {
            this.set_startup_menu(menu_id.clone(), cx);
        });

        let on_select_startup_filter = cx.listener(
            |this, filter: &Option<crate::entities::startup::StartupSource>, _window, cx| {
                this.set_startup_filter(*filter, cx);
            },
        );

        let on_change_startup_search = cx.listener(|this, query: &String, _window, cx| {
            this.set_startup_search_query(query.clone(), cx);
        });

        let on_hover_startup_search = cx.listener(|this, &hovered: &bool, _window, cx| {
            this.set_startup_search_hovered(hovered, cx);
        });

        let on_focus_startup_search = cx.listener(|this, &focused: &bool, _window, cx| {
            this.set_startup_search_focused(focused, cx);
        });

        let on_selection_startup_search =
            cx.listener(|this, selection: &Option<(usize, usize)>, _window, cx| {
                this.set_startup_search_selection(*selection, cx);
            });

        let on_hover_startup_card = cx.listener(|this, card_id: &Option<String>, _window, cx| {
            this.set_hovered_startup_card(card_id.clone(), cx);
        });

        let on_cleanup_toggle_target = cx.listener(|this, id: &String, _window, cx| {
            this.cleanup.toggle_target(id);
            cx.notify();
        });
        let on_cleanup_toggle_category =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                this.cleanup.toggle_category(*category);
                cx.notify();
            });
        let on_cleanup_toggle_expanded =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                this.cleanup.expanded =
                    (this.cleanup.expanded != Some(*category)).then_some(*category);
                cx.notify();
            });
        let on_cleanup_toggle_all = cx.listener(|this, _event: &(), _window, cx| {
            this.cleanup.toggle_all();
            cx.notify();
        });
        let on_cleanup_refresh = cx.listener(|this, _event: &(), _window, cx| {
            this.refresh_cleanup(cx);
        });
        let on_cleanup_clean =
            cx.listener(|this, category: &Option<CleanupCategory>, _window, cx| {
                this.clean_cleanup(*category, cx);
            });

        let minimize_to_tray = self.config.minimize_to_tray;
        let autostart = self.config.autostart;
        let autostart_to_tray = self.config.autostart_to_tray;
        let discord_rpc = self.config.discord_rpc;
        let startup_filter = self.startup_filter;
        let startup_open_menu_id = self.startup_open_menu_id.as_deref();
        let hovered_startup_card = self.hovered_startup_card.clone();
        let startup_search_focus = self
            .startup_search_focus
            .get_or_insert_with(|| cx.focus_handle())
            .clone();
        let cleanup_page = CleanupPage::new(
            self.cleanup.clone(),
            Rc::new(move |id, window, cx| {
                on_cleanup_toggle_target(&id, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_category(&category, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_expanded(&category, window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_toggle_all(&(), window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_refresh(&(), window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_clean(&category, window, cx);
            }),
        );

        let main_panel = div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme.main_bg)
            .border_t_1()
            .border_l_1()
            .border_color(theme.main_border)
            .rounded_tl(px(8.0))
            .overflow_hidden()
            .child(render_route(
                current_route,
                telemetry,
                windows_build,
                hovered_telemetry_card,
                current_locale,
                open_dropdown,
                self.open_dropdown_upward,
                closing_dropdown,
                hovered_dropdown,
                hovered_option,
                pending_selection,
                &self.gpu_engine_slots,
                minimize_to_tray,
                autostart,
                autostart_to_tray,
                discord_rpc,
                &self.startup_entries,
                startup_filter,
                &self.startup_search_query,
                self.startup_search_focused,
                self.startup_search_hovered,
                self.startup_search_selection,
                &startup_search_focus,
                startup_open_menu_id,
                hovered_startup_card,
                cleanup_page,
                move |target_route, window, cx| {
                    on_navigate_page(&target_route, window, cx);
                },
                move |card_id, is_hovered, window, cx| {
                    on_hover_telemetry_card(&(card_id, is_hovered), window, cx);
                },
                move |tweak_id, enabled, window, cx| {
                    on_toggle_tweak(&(tweak_id, enabled), window, cx);
                },
                move |preset, window, cx| {
                    on_change_keyboard_repeat(preset, window, cx);
                },
                move |preset, window, cx| {
                    on_change_ctf_optimization(preset, window, cx);
                },
                move |preset, window, cx| {
                    on_change_snapkey(preset, window, cx);
                },
                move |pal, window, cx| {
                    on_change_pal(pal, window, cx);
                },
                move |lang, window, cx| {
                    on_change_lang(lang, window, cx);
                },
                move |mode, window, cx| {
                    on_change_th(mode, window, cx);
                },
                move |enabled, window, cx| {
                    on_change_trans(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_min_tray(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_autostart(&enabled, window, cx);
                },
                move |enabled, window, cx| {
                    on_toggle_autostart_tray(&enabled, window, cx);
                },
                move |act, window, cx| {
                    on_change_disc(act, window, cx);
                },
                move |gpu_id, slot_idx, engine, window, cx| {
                    on_select_gpu_engine(&(gpu_id, slot_idx, engine), window, cx);
                },
                move |gpu_id, window, cx| {
                    on_reset_gpu_slots(&gpu_id, window, cx);
                },
                move |name, window, cx| {
                    on_toggle_drop(&name, window, cx);
                },
                move |name, &is_hovered, window, cx| {
                    on_hover_drop(&(name, is_hovered), window, cx);
                },
                move |dropdown, opt, &is_hovered, window, cx| {
                    on_hover_opt(&(dropdown, opt, is_hovered), window, cx);
                },
                move |window, cx| {
                    on_close_drop(&(), window, cx);
                },
                move |tt, window, cx| {
                    page_tooltip_listener(&tt, window, cx);
                },
                move |entry, window, cx| {
                    on_toggle_startup(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_delete_startup(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_open_startup_folder(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_open_startup_source(entry, window, cx);
                },
                move |entry, window, cx| {
                    on_copy_startup_path(entry, window, cx);
                },
                move |menu_id, window, cx| {
                    on_toggle_startup_menu(&menu_id, window, cx);
                },
                move |filter, window, cx| {
                    on_select_startup_filter(&filter, window, cx);
                },
                move |q, window, cx| {
                    on_change_startup_search(&q, window, cx);
                },
                move |hov, window, cx| {
                    on_hover_startup_search(hov, window, cx);
                },
                move |foc, window, cx| {
                    on_focus_startup_search(&foc, window, cx);
                },
                move |sel, window, cx| {
                    on_selection_startup_search(&sel, window, cx);
                },
                move |card_id, window, cx| {
                    on_hover_startup_card(&card_id, window, cx);
                },
            ));

        let content_row = div()
            .flex()
            .flex_row()
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .child(sidebar)
            .child(main_panel);

        let mut root = div()
            .font_family("IBM Plex Sans")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.window_bg)
            .child(titlebar)
            .child(content_row);

        if let Some(ref tt) = active_tooltip {
            root = root.child(Tooltip::new(tt.text.clone(), tt.cursor_pos));
        }

        if !self.toasts.is_empty() {
            let on_dismiss_toast = cx.listener(|this, toast_id: &str, _window, cx| {
                this.dismiss_toast(toast_id, cx);
            });
            let on_hover_toast_btn = cx.listener(
                |this, &(ref t_id, idx, is_hov): &(SharedString, usize, bool), _window, cx| {
                    this.set_hovered_toast_button(t_id, idx, is_hov, cx);
                },
            );
            let on_hover_stack = cx.listener(|this, &is_hov: &bool, _window, cx| {
                this.set_toast_stack_expanded(is_hov, cx);
            });

            let stack_el = crate::shared::ui::ToastStack::new(self.toasts.clone())
                .closing_id(self.closing_toast_id.clone())
                .hovered_toast_button(self.hovered_toast_button.clone())
                .expanded(self.toast_stack_expanded)
                .on_dismiss(move |toast_id, window, cx| {
                    on_dismiss_toast(toast_id, window, cx);
                })
                .on_hover_button(move |toast_id, idx, is_hov, window, cx| {
                    on_hover_toast_btn(&(toast_id.to_string().into(), idx, *is_hov), window, cx);
                })
                .on_hover_stack(move |is_hov, window, cx| {
                    on_hover_stack(is_hov, window, cx);
                })
                .into_any_element();

            root = root.child(gpui::deferred(stack_el).with_priority(200));
        }

        #[cfg(debug_assertions)]
        let (on_dev_move, on_dev_up) = {
            let on_dev_move = Arc::new(cx.listener(
                |this, event: &gpui::MouseMoveEvent, window, cx| {
                    if !event.dragging() {
                        if this.dev_perf_monitor.is_dragging {
                            this.dev_perf_monitor.end_drag();
                            cx.notify();
                        }
                        return;
                    }
                    let vp = window.viewport_size();
                    this.dev_perf_monitor
                        .update_drag(event.position, vp.width, vp.height);
                    cx.notify();
                },
            ));
            let on_dev_up = Arc::new(cx.listener(
                |this, _event: &gpui::MouseUpEvent, _window, cx| {
                    this.dev_perf_monitor.end_drag();
                    cx.notify();
                },
            ));
            (on_dev_move, on_dev_up)
        };

        #[cfg(debug_assertions)]
        if self.dev_perf_monitor.is_dragging {
            let move_cb = on_dev_move.clone();
            let up_cb = on_dev_up.clone();
            let up_cb_out = on_dev_up.clone();
            let up_cb_right = on_dev_up.clone();
            let up_cb_right_out = on_dev_up.clone();
            let down_cb = on_dev_up.clone();
            let down_cb_right = on_dev_up.clone();

            root = root
                .on_mouse_move({
                    let move_cb = on_dev_move.clone();
                    move |event, window, cx| {
                        move_cb(event, window, cx);
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let up_cb = on_dev_up.clone();
                    move |event, window, cx| {
                        up_cb(event, window, cx);
                    }
                })
                .on_mouse_up_out(MouseButton::Left, {
                    let up_cb = on_dev_up.clone();
                    move |event, window, cx| {
                        up_cb(event, window, cx);
                    }
                })
                .child(
                    div()
                        .id("dev_perf_drag_capture")
                        .absolute()
                        .inset_0()
                        .cursor_move()
                        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                            cx.stop_propagation();
                            down_cb(&gpui::MouseUpEvent::default(), window, cx);
                        })
                        .on_mouse_down(MouseButton::Right, move |_event, window, cx| {
                            cx.stop_propagation();
                            down_cb_right(&gpui::MouseUpEvent::default(), window, cx);
                        })
                        .on_mouse_move(move |event, window, cx| {
                            move_cb(event, window, cx);
                        })
                        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                            up_cb(event, window, cx);
                        })
                        .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                            up_cb_out(event, window, cx);
                        })
                        .on_mouse_up(MouseButton::Right, move |event, window, cx| {
                            up_cb_right(event, window, cx);
                        })
                        .on_mouse_up_out(MouseButton::Right, move |event, window, cx| {
                            up_cb_right_out(event, window, cx);
                        }),
                );
        }

        #[cfg(debug_assertions)]
        if self.dev_perf_monitor.enabled {
            let on_toggle_min = cx.listener(|this, _event: &(), _window, cx| {
                this.dev_perf_monitor.minimized = !this.dev_perf_monitor.minimized;
                cx.notify();
            });
            let on_freeze_tel = cx.listener(|this, _event: &(), _window, cx| {
                this.dev_perf_monitor.freeze_telemetry = !this.dev_perf_monitor.freeze_telemetry;
                cx.notify();
            });
            let on_chart_anim = cx.listener(|this, _event: &(), _window, cx| {
                this.dev_perf_monitor.disable_chart_animation =
                    !this.dev_perf_monitor.disable_chart_animation;
                cx.notify();
            });
            let on_start_drag = cx.listener(
                |this,
                 &(mouse_pos, current_widget_pos): &(Point<Pixels>, Point<Pixels>),
                 _window,
                 cx| {
                    this.dev_perf_monitor
                        .start_drag(mouse_pos, current_widget_pos);
                    cx.notify();
                },
            );
            let on_close_hud = cx.listener(|this, _event: &(), _window, cx| {
                this.dev_perf_monitor.enabled = false;
                cx.notify();
            });

            let on_continuous = cx.listener(|this, _event: &(), _window, cx| {
                this.dev_perf_monitor.continuous_mode = !this.dev_perf_monitor.continuous_mode;
                cx.notify();
            });

            let on_hover_perf_control = cx.listener(
                |this, &(ctrl, is_hovered): &(&'static str, bool), _window, cx| {
                    let new_ctrl = if is_hovered {
                        Some(ctrl)
                    } else if this.dev_perf_monitor.hovered_control == Some(ctrl) {
                        None
                    } else {
                        return;
                    };
                    if this.dev_perf_monitor.hovered_control != new_ctrl {
                        this.dev_perf_monitor.set_hovered_control(new_ctrl);
                        cx.notify();
                    }
                },
            );

            let on_dev_move_widget = on_dev_move.clone();
            let on_dev_up_widget = on_dev_up.clone();

            let perf_widget = crate::widgets::dev_perf_monitor::DevPerfMonitor::new(
                self.dev_perf_monitor.snapshot(),
                current_route,
                move |window, cx| {
                    on_toggle_min(&(), window, cx);
                },
                move |window, cx| {
                    on_freeze_tel(&(), window, cx);
                },
                move |window, cx| {
                    on_chart_anim(&(), window, cx);
                },
                move |window, cx| {
                    on_continuous(&(), window, cx);
                },
                move |mouse_pos, current_pos, window, cx| {
                    on_start_drag(&(mouse_pos, current_pos), window, cx);
                },
                move |mouse_pos, is_pressed, window, cx| {
                    let event = gpui::MouseMoveEvent {
                        position: mouse_pos,
                        pressed_button: if is_pressed {
                            Some(MouseButton::Left)
                        } else {
                            None
                        },
                        modifiers: gpui::Modifiers::default(),
                    };
                    on_dev_move_widget(&event, window, cx);
                },
                move |window, cx| {
                    let event = gpui::MouseUpEvent {
                        button: MouseButton::Left,
                        position: gpui::point(px(0.0), px(0.0)),
                        modifiers: gpui::Modifiers::default(),
                        click_count: 1,
                    };
                    on_dev_up_widget(&event, window, cx);
                },
                move |window, cx| {
                    on_close_hud(&(), window, cx);
                },
            )
            .on_hover_control(move |ctrl, is_hovered, window, cx| {
                on_hover_perf_control(&(ctrl, is_hovered), window, cx);
            });

            root = root.child(perf_widget);
        }

        #[cfg(debug_assertions)]
        {
            #[allow(clippy::cast_precision_loss)]
            let draw_ms = render_start.elapsed().as_secs_f32() * 1000.0;
            self.dev_perf_monitor.record_frame(draw_ms);
            if self.dev_perf_monitor.continuous_mode && self.dev_perf_monitor.enabled {
                window.request_animation_frame();
            }
        }

        root
    }
}
