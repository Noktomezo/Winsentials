use std::time::Duration;

use gpui::{Context, SharedString, Window};

use crate::entities::cleanup::CleanupCategory;

use super::AppView;

impl AppView {
    pub(crate) fn refresh_cleanup(&mut self, cx: &mut Context<Self>) {
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

    pub(crate) fn clean_cleanup(&mut self, category: Option<CleanupCategory>, cx: &mut Context<Self>) {
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

    pub fn show_toast(&mut self, mut toast: crate::shared::ui::ToastData, cx: &mut Context<Self>) {
        let duration = toast.duration;
        let toast_id = toast.id.clone();

        if let Some(existing) = self.toasts.iter_mut().find(|t| t.id == toast_id) {
            if toast.progress.is_none() && existing.progress.is_none() {
                existing.count += 1;
            }
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
}