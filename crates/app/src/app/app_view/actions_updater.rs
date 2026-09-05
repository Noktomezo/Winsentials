use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use gpui::Context;

use super::AppView;

impl AppView {
    #[allow(clippy::unused_self)]
    pub fn start_updater_polling(&mut self, cx: &mut Context<Self>) {
        let client = self.http_client.clone();
        cx.spawn(async move |this, cx| {
            // Initial delay to avoid competing with app launch
            cx.background_executor().timer(Duration::from_secs(3)).await;

            loop {
                let mut should_check = false;
                let _ = this.update(cx, |this, _cx| {
                    should_check = this.config.check_updates
                        && !matches!(
                            this.update_state,
                            crate::features::updater::UpdateState::Checking
                                | crate::features::updater::UpdateState::Downloading { .. }
                                | crate::features::updater::UpdateState::Installing { .. }
                        );
                });

                if should_check {
                    let _ = this.update(cx, |this, cx| {
                        this.update_state = crate::features::updater::UpdateState::Checking;
                        cx.notify();
                    });

                    let current_ver = crate::features::updater::CURRENT_VERSION;
                    let http_client = client.clone();
                    let tokio_handle = crate::shared::async_runtime::spawn_tokio(async move {
                        crate::features::updater::check_for_update(&http_client, current_ver).await
                    });
                    let res = tokio_handle
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r);

                    let update_res = this.update(cx, |this, cx| {
                        if let Ok(Some(info)) = res {
                            this.on_update_found(&info, cx);
                        } else {
                            this.update_state = crate::features::updater::UpdateState::UpToDate;
                            cx.notify();
                        }
                    });

                    if update_res.is_err() {
                        break;
                    }
                }

                cx.background_executor()
                    .timer(crate::features::updater::UPDATE_POLL_INTERVAL)
                    .await;
            }
        })
        .detach();
    }

    fn on_update_found(
        &mut self,
        info: &crate::features::updater::UpdateInfo,
        cx: &mut Context<Self>,
    ) {
        let is_already_available = matches!(
            &self.update_state,
            crate::features::updater::UpdateState::UpdateAvailable(curr) if curr.version == info.version
        );
        self.update_state = crate::features::updater::UpdateState::UpdateAvailable(info.clone());
        cx.notify();

        if !is_already_available {
            let on_dl = cx.listener(|this, (), _window, cx| {
                this.dismiss_toast("update_available_toast", cx);
                this.download_and_install_update(cx);
            });
            let dl_btn = crate::shared::ui::ToastButton::new(rust_i18n::t!(
                "settings.toast_update_download_btn"
            ))
            .variant(crate::shared::ui::ToastButtonVariant::Primary)
            .icon("icons/download.svg")
            .on_click(move |window, cx| {
                on_dl(&(), window, cx);
            });

            let on_later = cx.listener(|this, (), _window, cx| {
                this.dismiss_toast("update_available_toast", cx);
            });
            let later_btn = crate::shared::ui::ToastButton::new(rust_i18n::t!(
                "settings.toast_update_later_btn"
            ))
            .variant(crate::shared::ui::ToastButtonVariant::Secondary)
            .on_click(move |window, cx| {
                on_later(&(), window, cx);
            });

            let on_disable = cx.listener(|this, (), _window, cx| {
                this.dismiss_toast("update_available_toast", cx);
                this.toggle_check_updates(false, cx);
            });
            let disable_btn = crate::shared::ui::ToastButton::new(rust_i18n::t!(
                "settings.toast_update_disable_btn"
            ))
            .variant(crate::shared::ui::ToastButtonVariant::Outline)
            .full_width(true)
            .icon("icons/bell-off.svg")
            .on_click(move |window, cx| {
                on_disable(&(), window, cx);
            });

            let toast = crate::shared::ui::ToastData::new(
                "update_available_toast",
                rust_i18n::t!("settings.update_toast_title"),
            )
            .description(format!("Winsentials v{}", info.version))
            .variant(crate::shared::ui::ToastVariant::Info)
            .duration(Some(Duration::from_secs(16)))
            .button(dl_btn)
            .button(later_btn)
            .button(disable_btn);

            self.show_toast(toast, cx);
        }
    }

    pub fn check_for_updates(&mut self, manual: bool, cx: &mut Context<Self>) {
        if matches!(
            self.update_state,
            crate::features::updater::UpdateState::Checking
                | crate::features::updater::UpdateState::Downloading { .. }
                | crate::features::updater::UpdateState::Installing { .. }
        ) {
            return;
        }

        self.update_state = crate::features::updater::UpdateState::Checking;
        cx.notify();

        let client = self.http_client.clone();
        let current_version = crate::features::updater::CURRENT_VERSION;

        cx.spawn(async move |this, cx| {
            let tokio_handle = crate::shared::async_runtime::spawn_tokio(async move {
                crate::features::updater::check_for_update(&client, current_version).await
            });
            let res = tokio_handle
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r);

            let _ = this.update(cx, |this, cx| {
                match res {
                    Ok(Some(info)) => {
                        this.on_update_found(&info, cx);
                    }
                    Ok(None) => {
                        this.update_state = crate::features::updater::UpdateState::UpToDate;
                    }
                    Err(err) => {
                        if manual {
                            this.update_state = crate::features::updater::UpdateState::Error(err);
                        } else {
                            this.update_state = crate::features::updater::UpdateState::UpToDate;
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    #[allow(clippy::too_many_lines)]
    pub fn download_and_install_update(&mut self, cx: &mut Context<Self>) {
        let crate::features::updater::UpdateState::UpdateAvailable(info) = &self.update_state
        else {
            return;
        };
        let info = info.clone();
        let version = info.version.clone();
        self.update_state = crate::features::updater::UpdateState::Downloading {
            version: version.clone(),
            progress: 0.0,
        };
        self.dismiss_toast("update_available_toast", cx);

        let initial_toast = crate::shared::ui::ToastData::new(
            "updater_download_progress",
            rust_i18n::t!("settings.update_toast_downloading").to_string(),
        )
        .description(format!("Winsentials v{version}"))
        .variant(crate::shared::ui::ToastVariant::Info)
        .progress(Some(crate::shared::ui::ToastProgress {
            value: 0.0,
            label: Some("0%".into()),
        }));
        self.show_toast(initial_toast, cx);
        cx.notify();

        let client = self.http_client.clone();
        let progress_atomic = Arc::new(AtomicU32::new(0));
        let is_done = Arc::new(AtomicBool::new(false));

        let prog_for_download = Arc::clone(&progress_atomic);
        let done_for_download = Arc::clone(&is_done);

        let download_handle = crate::shared::async_runtime::spawn_tokio(async move {
            let res =
                crate::features::updater::download_and_install_update(&client, &info, move |p| {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let permille = (p * 1000.0).clamp(0.0, 1000.0) as u32;
                    prog_for_download.store(permille, Ordering::Relaxed);
                })
                .await;
            done_for_download.store(true, Ordering::Relaxed);
            res
        });

        let prog_for_ui = Arc::clone(&progress_atomic);
        let done_for_ui = Arc::clone(&is_done);

        cx.spawn(async move |this, cx| {
            while !done_for_ui.load(Ordering::Relaxed) {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                #[allow(clippy::cast_precision_loss)]
                let p = prog_for_ui.load(Ordering::Relaxed) as f32 / 1000.0;
                let res = this.update(cx, |this, cx| {
                    if let crate::features::updater::UpdateState::Downloading {
                        ref mut progress,
                        ref version,
                    } = this.update_state
                    {
                        *progress = p;
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let pct_int = (p * 100.0).clamp(0.0, 100.0) as u32;
                        let progress_toast = crate::shared::ui::ToastData::new(
                            "updater_download_progress",
                            rust_i18n::t!("settings.update_toast_downloading").to_string(),
                        )
                        .description(format!("Winsentials v{version}"))
                        .variant(crate::shared::ui::ToastVariant::Info)
                        .progress(Some(crate::shared::ui::ToastProgress {
                            value: p,
                            label: Some(format!("{pct_int}%").into()),
                        }));
                        this.show_toast(progress_toast, cx);
                        cx.notify();
                    }
                });
                if res.is_err() {
                    break;
                }
            }

            let download_result = download_handle
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r);

            let is_success = download_result.is_ok();
            let _ = is_success;
            let _ = this.update(cx, |this, cx| {
                match download_result {
                    Ok(()) => {
                        this.update_state = crate::features::updater::UpdateState::Installing {
                            version: version.clone(),
                        };

                        let ready_toast = crate::shared::ui::ToastData::new(
                            "updater_download_progress",
                            rust_i18n::t!("settings.update_toast_ready").to_string(),
                        )
                        .description(format!("Winsentials v{version}"))
                        .variant(crate::shared::ui::ToastVariant::Success)
                        .progress(Some(crate::shared::ui::ToastProgress {
                            value: 1.0,
                            label: Some(
                                rust_i18n::t!("settings.update_restarting")
                                    .to_string()
                                    .into(),
                            ),
                        }));
                        this.show_toast(ready_toast, cx);
                    }
                    Err(err) => {
                        this.update_state =
                            crate::features::updater::UpdateState::Error(err.clone());
                        let err_toast = crate::shared::ui::ToastData::new(
                            "updater_download_progress",
                            rust_i18n::t!("settings.status_error").to_string(),
                        )
                        .description(err)
                        .variant(crate::shared::ui::ToastVariant::Error)
                        .duration(Some(Duration::from_secs(6)));
                        this.show_toast(err_toast, cx);
                    }
                }
                cx.notify();
            });

            #[cfg(all(windows, debug_assertions))]
            if is_success {
                cx.background_executor()
                    .timer(Duration::from_millis(800))
                    .await;
                if let Ok(current_exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(current_exe).spawn();
                    std::process::exit(0);
                }
            }
        })
        .detach();
    }

    pub fn toggle_check_updates(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.config.check_updates = enabled;
        let _ = crate::entities::save_config(&self.config);
        if enabled {
            self.check_for_updates(false, cx);
        }
        cx.notify();
    }
}