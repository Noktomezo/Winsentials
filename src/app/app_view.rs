use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px};

use crate::entities::{AppConfig, TelemetryData, load_config, save_config};
use crate::features::discord_rpc::{DiscordRpcActivity, DiscordRpcManager};
use crate::features::navigation::AppRoute;
use crate::features::tray::TrayManager;
use crate::pages::render_route;
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

        let config = load_config();

        let mut discord_manager = DiscordRpcManager::new(windows_build);
        if config.discord_rpc != DiscordRpcActivity::Disabled {
            discord_manager.set_activity(config.discord_rpc);
        }
        let discord_rpc_manager = Arc::new(Mutex::new(discord_manager));

        let tray_manager = TrayManager::new();
        let open_item_id = tray_manager.open_item_id.clone();
        let quit_item_id = tray_manager.quit_item_id.clone();
        let startup_entries = crate::entities::startup::fetch_all_startup_entries();

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
            cx.notify();
        }
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
            let _ = (tweak.set_applied)(enabled);
            crate::shared::shell_notify::notify_shell_change();
            if let Ok(mut mgr) = self.discord_rpc_manager.lock() {
                mgr.refresh_presence();
            }
            match tweak.restart {
                crate::entities::tweaks::RestartRequirement::Explorer => {
                    self.show_explorer_restart_toast(cx);
                }
                crate::entities::tweaks::RestartRequirement::Logoff => {
                    self.show_logoff_toast(cx);
                }
                crate::entities::tweaks::RestartRequirement::Reboot => {
                    self.show_reboot_toast(cx);
                }
                crate::entities::tweaks::RestartRequirement::None => {}
            }
            cx.notify();
        }
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
            other => Box::leak(other.to_string().into_boxed_str()),
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
            this.update(cx, |this, cx| {
                if dropdown_copy == "palette" {
                    this.set_palette(val_copy, cx);
                } else if dropdown_copy == "theme" {
                    this.set_theme_mode(val_copy, cx);
                } else if dropdown_copy == "language" {
                    this.set_language(val_copy, cx);
                } else if dropdown_copy == "transparency" {
                    this.set_transparency(val_copy == "enabled", cx);
                }
                this.pending_selection = None;
                this.start_closing_dropdown(dropdown_copy, cx);
            })
            .ok();
        })
        .detach();
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
            self.open_dropdown_upward = space_below < px(220.0) && space_above > space_below;
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
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                sidebar_expanded,
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
                move |target_route, window, cx| {
                    on_navigate_page(&target_route, window, cx);
                },
                move |card_id, is_hovered, window, cx| {
                    on_hover_telemetry_card(&(card_id, is_hovered), window, cx);
                },
                move |tweak_id, enabled, window, cx| {
                    on_toggle_tweak(&(tweak_id, enabled), window, cx);
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

        root
    }
}
