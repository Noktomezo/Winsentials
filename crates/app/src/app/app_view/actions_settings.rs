use gpui::{Context, Window};

use crate::entities::save_config;
use crate::features::discord_rpc::DiscordRpcActivity;
use crate::shared::theme::{Theme, ThemeMode, ThemePalette};

use super::AppView;

impl AppView {
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
            let is_applied_fn = tweak.is_applied;
            let restart = tweak.restart;
            let category = tweak.category;

            // 1. Optimistic immediate update: instantly update UI state so switch animates with 0 latency
            let mut tweak_states = if cx.has_global::<crate::entities::tweaks::TweakStates>() {
                cx.global::<crate::entities::tweaks::TweakStates>().clone()
            } else {
                crate::entities::tweaks::TweakStates::load_initial()
            };
            tweak_states.set_state(tweak_id, enabled);
            cx.set_global(tweak_states);
            cx.notify();

            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let res = set_applied(enabled);
                        if res.is_ok()
                            && matches!(
                                category,
                                crate::entities::tweaks::TweakCategory::Explorer
                                    | crate::entities::tweaks::TweakCategory::ContextMenu
                            )
                        {
                            crate::shared::shell_notify::notify_shell_change();
                        }
                        res
                    })
                    .await;
                if let Err(update_error) = this.update(cx, move |this, cx| {
                    let actual_applied = is_applied_fn();
                    let mut states = if cx.has_global::<crate::entities::tweaks::TweakStates>() {
                        cx.global::<crate::entities::tweaks::TweakStates>().clone()
                    } else {
                        crate::entities::tweaks::TweakStates::load_initial()
                    };
                    states.set_state(tweak_id, actual_applied);
                    cx.set_global(states);

                    match result {
                        Ok(()) => {
                            let discord_rpc = this.discord_rpc_manager.clone();
                            cx.background_executor()
                                .spawn(async move {
                                    if let Ok(mut mgr) = discord_rpc.lock() {
                                        mgr.refresh_presence();
                                    }
                                })
                                .detach();

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
                        Err(error) => {
                            this.show_setting_error(tweak_id, &error, cx);
                            cx.notify();
                        }
                    }
                }) {
                    eprintln!(
                        "failed to update tweak state after applying {tweak_id}: {update_error}"
                    );
                }
            })
            .detach();
        }
    }

    pub(crate) fn show_setting_error(&mut self, setting: &str, error: &str, cx: &mut Context<Self>) {
        eprintln!("failed to apply {setting}: {error}");
        let toast = crate::shared::ui::ToastData::new(
            "setting_apply_error",
            rust_i18n::t!("tweaks.apply_failed_title"),
        )
        .description(rust_i18n::t!("tweaks.apply_failed_desc"))
        .variant(crate::shared::ui::ToastVariant::Error);
        self.show_toast(toast, cx);
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
}