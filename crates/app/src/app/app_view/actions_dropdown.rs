use std::time::Duration;

use gpui::{Context, Window, px};

use super::AppView;

impl AppView {
    pub fn start_closing_dropdown(&mut self, name: &'static str, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(name) {
            self.open_dropdown = None;
            self.opening_dropdown = None;
            self.closing_dropdown = Some(name);
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(140))
                    .await;
                this.update(cx, |this, cx| {
                    if this.closing_dropdown == Some(name) {
                        this.closing_dropdown = None;
                        this.hovered_option = None;
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

        self.pending_selection = Some((dropdown, static_val));
        cx.notify();

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
            // Dropdown is currently playing closing animation; ignore click
        } else {
            let mouse_y = window.mouse_position().y;
            let viewport_h = window.viewport_size().height;
            let space_below = viewport_h - mouse_y;
            let space_above = mouse_y - px(40.0);
            let required_space = Self::dropdown_required_space_below(name);
            self.open_dropdown_upward = space_below < required_space && space_above > space_below;
            self.open_dropdown = Some(name);
            self.opening_dropdown = Some(name);
            self.closing_dropdown = None;
            cx.notify();

            let opening_name = name;
            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(160))
                    .await;
                this.update(cx, |this, cx| {
                    if this.opening_dropdown == Some(opening_name) {
                        this.opening_dropdown = None;
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
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
            self.start_closing_dropdown(open, cx);
        }
    }
}