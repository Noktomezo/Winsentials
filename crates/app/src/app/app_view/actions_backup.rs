use std::collections::HashMap;
use std::sync::Arc;

use gpui::Context;

use super::{AppView, ConfirmModalState, InputModalState};
use crate::entities::tweaks::backup::{
    create_backup_from_states, delete_backup, rename_backup, save_backups,
};
use crate::entities::tweaks::{ALL_TWEAKS, TweakCategory, TweakStates};

impl AppView {
    pub fn open_create_backup_modal(&mut self, cx: &mut Context<Self>) {
        let default_name = rust_i18n::t!("tools.create_backup_placeholder").to_string();

        let on_confirm = cx.listener(|this, name: &String, _window, cx| {
            let backup_name = if name.trim().is_empty() {
                rust_i18n::t!("tools.create_backup_placeholder").to_string()
            } else {
                name.trim().to_string()
            };
            this.close_input_modal(cx);
            this.execute_create_backup(&backup_name, cx);
        });

        let on_cancel = cx.listener(|this, _event: &(), _window, cx| {
            this.close_input_modal(cx);
        });

        self.input_modal = Some(InputModalState {
            title: rust_i18n::t!("tools.create_backup_title")
                .to_string()
                .into(),
            description: rust_i18n::t!("tools.create_backup_desc").to_string().into(),
            value: default_name,
            placeholder: rust_i18n::t!("tools.create_backup_placeholder")
                .to_string()
                .into(),
            confirm_label: rust_i18n::t!("tools.create_backup").to_string().into(),
            cancel_label: rust_i18n::t!("cleanup.cancel").to_string().into(),
            focused: true,
            selection: None,
            closing: false,
            on_confirm: Arc::new(move |val, window, cx| {
                on_confirm(&val, window, cx);
            }),
            on_cancel: Arc::new(move |window, cx| {
                on_cancel(&(), window, cx);
            }),
        });
        cx.notify();
    }

    pub fn execute_create_backup(&mut self, name: &str, cx: &mut Context<Self>) {
        let current_states = if cx.has_global::<TweakStates>() {
            let global_states = cx.global::<TweakStates>();
            let mut map = HashMap::with_capacity(ALL_TWEAKS.len());
            for tweak in ALL_TWEAKS {
                map.insert(tweak.id, global_states.is_applied(tweak));
            }
            map
        } else {
            let mut map = HashMap::with_capacity(ALL_TWEAKS.len());
            for tweak in ALL_TWEAKS {
                map.insert(tweak.id, (tweak.is_applied)());
            }
            map
        };

        let backup = create_backup_from_states(name, &current_states);
        let backup_name = backup.name.clone();
        self.tweak_backups.insert(0, backup);
        let _ = save_backups(&self.tweak_backups);

        let msg = rust_i18n::t!("tools.toast_backup_created", name = backup_name).to_string();
        self.show_toast(
            crate::shared::ui::ToastData::new("backup_created", msg)
                .variant(crate::shared::ui::ToastVariant::Success),
            cx,
        );
        cx.notify();
    }

    pub fn open_rename_backup_modal(&mut self, backup_id: String, cx: &mut Context<Self>) {
        let current_name = self
            .tweak_backups
            .iter()
            .find(|b| b.id == backup_id)
            .map_or_else(String::new, |b| b.name.clone());

        let id_for_confirm = backup_id;
        let on_confirm = cx.listener(move |this, name: &String, _window, cx| {
            let new_name = name.trim();
            if !new_name.is_empty() {
                this.execute_rename_backup(&id_for_confirm, new_name, cx);
            }
            this.close_input_modal(cx);
        });

        let on_cancel = cx.listener(|this, _event: &(), _window, cx| {
            this.close_input_modal(cx);
        });

        self.input_modal = Some(InputModalState {
            title: rust_i18n::t!("tools.rename_backup_title")
                .to_string()
                .into(),
            description: rust_i18n::t!("tools.rename_backup_desc").to_string().into(),
            value: current_name,
            placeholder: rust_i18n::t!("tools.create_backup_placeholder")
                .to_string()
                .into(),
            confirm_label: rust_i18n::t!("tools.save").to_string().into(),
            cancel_label: rust_i18n::t!("cleanup.cancel").to_string().into(),
            focused: true,
            selection: None,
            closing: false,
            on_confirm: Arc::new(move |val, window, cx| {
                on_confirm(&val, window, cx);
            }),
            on_cancel: Arc::new(move |window, cx| {
                on_cancel(&(), window, cx);
            }),
        });
        cx.notify();
    }

    pub fn execute_rename_backup(&mut self, id: &str, new_name: &str, cx: &mut Context<Self>) {
        if rename_backup(id, new_name, &mut self.tweak_backups) {
            let msg = rust_i18n::t!("tools.toast_backup_renamed", name = new_name).to_string();
            self.show_toast(
                crate::shared::ui::ToastData::new("backup_renamed", msg)
                    .variant(crate::shared::ui::ToastVariant::Success),
                cx,
            );
            cx.notify();
        }
    }

    pub fn confirm_delete_backup(&mut self, backup_id: String, cx: &mut Context<Self>) {
        let backup_name = self
            .tweak_backups
            .iter()
            .find(|b| b.id == backup_id)
            .map_or_else(|| "Backup".to_string(), |b| b.name.clone());

        let id_for_delete = backup_id;
        let name_for_toast = backup_name.clone();

        let on_confirm = cx.listener(move |this, _event: &(), _window, cx| {
            this.close_confirm_modal(cx);
            if delete_backup(&id_for_delete, &mut this.tweak_backups) {
                let msg =
                    rust_i18n::t!("tools.toast_backup_deleted", name = name_for_toast).to_string();
                this.show_toast(
                    crate::shared::ui::ToastData::new("backup_deleted", msg)
                        .variant(crate::shared::ui::ToastVariant::Default),
                    cx,
                );
                cx.notify();
            }
        });

        let on_cancel = cx.listener(|this, _event: &(), _window, cx| {
            this.close_confirm_modal(cx);
        });

        self.confirm_modal = Some(ConfirmModalState {
            title: rust_i18n::t!("tools.delete_backup_title")
                .to_string()
                .into(),
            description: rust_i18n::t!("tools.delete_backup_desc", name = backup_name)
                .to_string()
                .into(),
            confirm_label: rust_i18n::t!("tools.delete_backup").to_string().into(),
            cancel_label: rust_i18n::t!("cleanup.cancel").to_string().into(),
            is_destructive: true,
            closing: false,
            on_confirm: Arc::new(move |window, cx| {
                on_confirm(&(), window, cx);
            }),
            on_cancel: Arc::new(move |window, cx| {
                on_cancel(&(), window, cx);
            }),
            on_close: None,
        });
        cx.notify();
    }

    pub fn confirm_restore_backup(&mut self, backup_id: String, cx: &mut Context<Self>) {
        let backup_name = self
            .tweak_backups
            .iter()
            .find(|b| b.id == backup_id)
            .map_or_else(|| "Backup".to_string(), |b| b.name.clone());

        let id_for_restore = backup_id;
        let on_confirm = cx.listener(move |this, _event: &(), _window, cx| {
            this.close_confirm_modal(cx);
            this.execute_restore_backup(&id_for_restore, cx);
        });

        let on_cancel = cx.listener(|this, _event: &(), _window, cx| {
            this.close_confirm_modal(cx);
        });

        self.confirm_modal = Some(ConfirmModalState {
            title: rust_i18n::t!("tools.restore_backup_title")
                .to_string()
                .into(),
            description: rust_i18n::t!("tools.restore_backup_desc", name = backup_name)
                .to_string()
                .into(),
            confirm_label: rust_i18n::t!("tools.restore_backup").to_string().into(),
            cancel_label: rust_i18n::t!("cleanup.cancel").to_string().into(),
            is_destructive: false,
            closing: false,
            on_confirm: Arc::new(move |window, cx| {
                on_confirm(&(), window, cx);
            }),
            on_cancel: Arc::new(move |window, cx| {
                on_cancel(&(), window, cx);
            }),
            on_close: None,
        });
        cx.notify();
    }

    #[allow(clippy::too_many_lines)]
    pub fn execute_restore_backup(&mut self, backup_id: &str, cx: &mut Context<Self>) {
        let Some(backup) = self
            .tweak_backups
            .iter()
            .find(|b| b.id == backup_id)
            .cloned()
        else {
            return;
        };

        let mut needs_shell_refresh = false;
        let mut max_restart = crate::entities::tweaks::RestartRequirement::None;

        let mut current_states = if cx.has_global::<TweakStates>() {
            cx.global::<TweakStates>().clone()
        } else {
            TweakStates::load_initial()
        };

        for tweak in ALL_TWEAKS {
            if let Some(&desired) = backup.tweak_states.get(tweak.id) {
                let current = current_states.is_applied(tweak);
                if current != desired {
                    let res = (tweak.set_applied)(desired);
                    if res.is_ok() {
                        current_states.set_state(tweak.id, desired);
                        if matches!(
                            tweak.category,
                            TweakCategory::Explorer | TweakCategory::ContextMenu
                        ) {
                            needs_shell_refresh = true;
                        }
                        if tweak.restart > max_restart {
                            max_restart = tweak.restart;
                        }
                    }
                }
            }
        }

        cx.set_global(current_states);

        if needs_shell_refresh {
            crate::shared::shell_notify::notify_shell_change();
        }

        match max_restart {
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

        let msg = rust_i18n::t!("tools.toast_backup_restored", name = backup.name).to_string();
        self.show_toast(
            crate::shared::ui::ToastData::new("backup_restored", msg)
                .variant(crate::shared::ui::ToastVariant::Success),
            cx,
        );
        cx.notify();
    }
}
