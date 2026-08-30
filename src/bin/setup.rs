#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![allow(
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unnecessary_wraps,
    clippy::unused_self
)]

use std::path::{Path, PathBuf};

use gpui::{
    AnimationExt, App, AppContext, Application, Bounds, Context, Div, FontWeight,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, Render,
    SharedString, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled,
    TitlebarOptions, WeakEntity, WindowBackgroundAppearance, WindowBounds, WindowControlArea,
    WindowOptions, div, img, point, px, rgb, rgba, size,
};
use winsentials::shared::assets::EmbeddedAssetSource;
use winsentials::shared::theme::{Theme, arclate};
use winsentials::shared::ui::{Icon, Switch};
use winsentials::widgets::sidebar::{lerp_item_bg, lerp_item_text};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const APP_VERSION: &str = "0.1.0";
const PAYLOAD_BYTES: &[u8] = include_bytes!("../../target/release/Winsentials.exe");

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetupStep {
    Welcome,
    Installing,
    Finished,
    Error,
    UninstallConfirm,
    Uninstalling,
    Uninstalled,
}

struct SetupView {
    step: SetupStep,
    install_dir: PathBuf,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
    launch_after: bool,
    status_text: SharedString,
    error_text: Option<SharedString>,
    installing: bool,
    hovered_win_control: Option<&'static str>,
}

impl SetupView {
    fn new(is_uninstall: bool) -> Self {
        let default_dir = std::env::var("ProgramFiles").map_or_else(
            |_| PathBuf::from(r"C:\Program Files\Winsentials"),
            |pf| PathBuf::from(pf).join("Winsentials"),
        );

        Self {
            step: if is_uninstall {
                SetupStep::UninstallConfirm
            } else {
                SetupStep::Welcome
            },
            install_dir: default_dir,
            desktop_shortcut: true,
            start_menu_shortcut: true,
            launch_after: true,
            status_text: SharedString::from("Подготовка к установке..."),
            error_text: None,
            installing: false,
            hovered_win_control: None,
        }
    }

    fn select_directory(&mut self, cx: &mut Context<Self>) {
        let current = self.install_dir.clone();
        if let Some(folder) = rfd::FileDialog::new()
            .set_directory(&current)
            .set_title("Выберите папку установки Winsentials")
            .pick_folder()
        {
            self.install_dir = folder;
            cx.notify();
        }
    }

    fn start_installation(&mut self, cx: &mut Context<Self>) {
        if self.installing {
            return;
        }
        self.installing = true;
        self.step = SetupStep::Installing;
        self.status_text = SharedString::from("Завершение запущенных процессов...");
        cx.notify();

        let install_dir = self.install_dir.clone();
        let desktop_shortcut = self.desktop_shortcut;
        let start_menu_shortcut = self.start_menu_shortcut;

        cx.spawn(
            move |this: WeakEntity<SetupView>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    let res = cx
                        .background_executor()
                        .spawn(async move {
                            run_install_pipeline(
                                &install_dir,
                                desktop_shortcut,
                                start_menu_shortcut,
                            )
                        })
                        .await;

                    let _ = this.update(&mut cx, |view, cx| {
                        view.installing = false;
                        match res {
                            Ok(()) => {
                                view.step = SetupStep::Finished;
                                view.status_text =
                                    SharedString::from("Установка Winsentials успешно завершена!");
                            }
                            Err(err) => {
                                view.step = SetupStep::Error;
                                view.error_text = Some(SharedString::from(err));
                            }
                        }
                        cx.notify();
                    });
                }
            },
        )
        .detach();
    }

    fn start_uninstallation(&mut self, cx: &mut Context<Self>) {
        if self.installing {
            return;
        }
        self.installing = true;
        self.step = SetupStep::Uninstalling;
        self.status_text = SharedString::from("Удаление Winsentials...");
        cx.notify();

        let install_dir = self.install_dir.clone();

        cx.spawn(
            move |this: WeakEntity<SetupView>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    let res = cx
                        .background_executor()
                        .spawn(async move { run_uninstall_pipeline(&install_dir) })
                        .await;

                    let _ = this.update(&mut cx, |view, cx| {
                        view.installing = false;
                        match res {
                            Ok(()) => {
                                view.step = SetupStep::Uninstalled;
                                view.status_text = SharedString::from(
                                    "Winsentials успешно удален с вашего компьютера.",
                                );
                            }
                            Err(err) => {
                                view.step = SetupStep::Error;
                                view.error_text = Some(SharedString::from(err));
                            }
                        }
                        cx.notify();
                    });
                }
            },
        )
        .detach();
    }
}

fn run_install_pipeline(
    install_dir: &Path,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
) -> Result<(), String> {
    // 1. Terminate running instances
    terminate_winsentials();
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 2. Create install directory
    std::fs::create_dir_all(install_dir)
        .map_err(|e| format!("Не удалось создать папку установки: {e}"))?;

    let target_exe = install_dir.join("Winsentials.exe");

    // 3. Write target executable from embedded payload or current binary
    if !PAYLOAD_BYTES.is_empty() {
        std::fs::write(&target_exe, PAYLOAD_BYTES)
            .map_err(|e| format!("Не удалось записать Winsentials.exe: {e}"))?;
    } else if let Ok(current_exe) = std::env::current_exe() {
        let _ = std::fs::copy(&current_exe, &target_exe);
    }

    // 4. Create Desktop Shortcut
    if desktop_shortcut {
        if let Some(desktop) = get_desktop_dir() {
            let lnk = desktop.join("Winsentials.lnk");
            let _ = create_windows_shortcut(&lnk, &target_exe);
        }
    }

    // 5. Create Start Menu Shortcut
    if start_menu_shortcut {
        if let Some(programs) = get_start_menu_programs_dir() {
            let dir = programs.join("Winsentials");
            let _ = std::fs::create_dir_all(&dir);
            let lnk = dir.join("Winsentials.lnk");
            let _ = create_windows_shortcut(&lnk, &target_exe);

            // Uninstall shortcut
            let uninst_lnk = dir.join("Удалить Winsentials.lnk");
            let _ = create_windows_shortcut_args(&uninst_lnk, &target_exe, "--uninstall");
        }
    }

    // 6. Write Add/Remove Programs Registry Keys
    register_uninstall_registry(install_dir, &target_exe, APP_VERSION);

    Ok(())
}

fn run_uninstall_pipeline(install_dir: &Path) -> Result<(), String> {
    terminate_winsentials();
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 1. Delete shortcuts
    if let Some(desktop) = get_desktop_dir() {
        let _ = std::fs::remove_file(desktop.join("Winsentials.lnk"));
    }
    if let Some(programs) = get_start_menu_programs_dir() {
        let dir = programs.join("Winsentials");
        let _ = std::fs::remove_dir_all(dir);
    }

    // 2. Remove Registry entries
    unregister_uninstall_registry();

    // 3. Remove files and directory
    let exe = install_dir.join("Winsentials.exe");
    let _ = std::fs::remove_file(exe);
    let _ = std::fs::remove_dir(install_dir);

    Ok(())
}

fn terminate_winsentials() {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "Winsentials.exe", "/T"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

fn get_desktop_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .ok()
        .map(|p| PathBuf::from(p).join("Desktop"))
        .or_else(|| {
            std::env::var("PUBLIC")
                .ok()
                .map(|p| PathBuf::from(p).join("Desktop"))
        })
}

fn get_start_menu_programs_dir() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"))
        .or_else(|| {
            std::env::var("ProgramData")
                .ok()
                .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"))
        })
}

fn create_windows_shortcut(lnk_path: &Path, target_exe: &Path) -> std::io::Result<()> {
    create_windows_shortcut_args(lnk_path, target_exe, "")
}

fn create_windows_shortcut_args(
    lnk_path: &Path,
    target_exe: &Path,
    args: &str,
) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = '{}'; $s.IconLocation = '{},0'; $s.WorkingDirectory = '{}'; $s.Save()",
            lnk_path.to_string_lossy().replace('\'', "''"),
            target_exe.to_string_lossy().replace('\'', "''"),
            args.replace('\'', "''"),
            target_exe.to_string_lossy().replace('\'', "''"),
            target_exe
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
                .replace('\'', "''")
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    let _ = (lnk_path, target_exe, args);
    Ok(())
}

fn register_uninstall_registry(install_dir: &Path, target_exe: &Path, version: &str) {
    if let Ok(key) = windows_registry::LOCAL_MACHINE
        .create(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials")
    {
        let _ = key.set_string("DisplayName", "Winsentials");
        let _ = key.set_string("DisplayVersion", version);
        let _ = key.set_string("Publisher", "Noktomezo");
        let _ = key.set_string("DisplayIcon", format!("{},0", target_exe.to_string_lossy()));
        let _ = key.set_string(
            "UninstallString",
            format!(r#""{}" --uninstall"#, target_exe.to_string_lossy()),
        );
        let _ = key.set_string(
            "QuietUninstallString",
            format!(r#""{}" --uninstall /quiet"#, target_exe.to_string_lossy()),
        );
        let _ = key.set_string("InstallLocation", install_dir.to_string_lossy());
        let _ = key.set_string("URLInfoAbout", "https://github.com/Noktomezo/Winsentials");
        let _ = key.set_u32("NoModify", 1);
        let _ = key.set_u32("NoRepair", 1);
    }
}

fn unregister_uninstall_registry() {
    let _ = windows_registry::LOCAL_MACHINE
        .remove_tree(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Winsentials");
    let _ = windows_registry::LOCAL_MACHINE.remove_tree(r"Software\Winsentials");
}

impl Render for SetupView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.step {
            SetupStep::Welcome => self.render_welcome(cx),
            SetupStep::Installing => self.render_installing(cx),
            SetupStep::Finished => self.render_finished(cx),
            SetupStep::Error => self.render_error(cx),
            SetupStep::UninstallConfirm => self.render_uninstall_confirm(cx),
            SetupStep::Uninstalling => self.render_uninstalling(cx),
            SetupStep::Uninstalled => self.render_uninstalled(cx),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(arclate::BG_DARK))
            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
            .font_family("IBM Plex Sans")
            .border_1()
            .border_color(rgb(arclate::BORDER_MAIN_DARK))
            .rounded(px(10.0))
            .overflow_hidden()
            .child(self.render_titlebar(cx))
            .child(div().flex_1().flex().flex_col().p(px(16.0)).child(content))
    }
}

impl SetupView {
    fn render_titlebar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let is_uninst = matches!(
            self.step,
            SetupStep::UninstallConfirm | SetupStep::Uninstalling | SetupStep::Uninstalled
        );

        let title_str = if is_uninst {
            "Удаление Winsentials"
        } else {
            "Установка Winsentials"
        };

        let is_min_hovered = self.hovered_win_control == Some("min");
        let is_close_hovered = self.hovered_win_control == Some("close");

        let min_target: f32 = if is_min_hovered { 0.5 } else { 0.0 };
        let close_target: f32 = if is_close_hovered { 0.5 } else { 0.0 };

        let min_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(min_target)
            .with_epsilon(0.005);
        let close_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
            .to(close_target)
            .with_epsilon(0.005);

        let accent_blue = theme.accent_blue;
        let theme_ref = theme;

        let min_text_color = lerp_item_text(&theme_ref, min_target);
        let close_text_color = if close_target > 0.0 {
            theme_ref.accent_red
        } else {
            theme_ref.text_primary
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(40.0))
            .p(px(4.0))
            .bg(rgb(arclate::BG_DARK))
            .border_b_1()
            .border_color(rgb(arclate::BORDER_MAIN_DARK))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .pl(px(8.0))
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(MouseButton::Left, |_ev, window, _cx| {
                        window.start_window_move();
                    })
                    .child(img("app-logo.png").size(px(18.0)).rounded(px(4.0)))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                            .child(title_str),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .id("win_minimize")
                            .window_control_area(WindowControlArea::Min)
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(32.0))
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .active(move |s| s.bg(theme.accent_active_bg))
                            .on_hover(cx.listener(|this, &hov: &bool, _window, cx| {
                                this.hovered_win_control = if hov { Some("min") } else { None };
                                cx.notify();
                            }))
                            .on_mouse_down(MouseButton::Left, |_ev, window, _cx| {
                                window.minimize_window();
                            })
                            .with_spring("win_min_spring", min_spring, move |btn, val| {
                                let bg = lerp_item_bg(accent_blue, val);
                                btn.bg(bg)
                            })
                            .child(
                                Icon::new("icons/minus.svg")
                                    .size(px(14.0))
                                    .color(min_text_color),
                            ),
                    )
                    .child(
                        div()
                            .id("win_close")
                            .window_control_area(WindowControlArea::Close)
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(32.0))
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .active(move |s| s.bg(theme.accent_red))
                            .on_hover(cx.listener(|this, &hov: &bool, _window, cx| {
                                this.hovered_win_control = if hov { Some("close") } else { None };
                                cx.notify();
                            }))
                            .on_mouse_down(MouseButton::Left, |_ev, _window, cx| {
                                cx.quit();
                            })
                            .with_spring("win_close_spring", close_spring, move |btn, val| {
                                let bg = lerp_item_bg(theme_ref.accent_red, val);
                                btn.bg(bg)
                            })
                            .child(
                                Icon::new("icons/x.svg")
                                    .size(px(14.0))
                                    .color(close_text_color),
                            ),
                    ),
            )
    }

    fn render_welcome(&self, cx: &mut Context<Self>) -> Div {
        let dir_str = self.install_dir.to_string_lossy().to_string();

        div()
            .flex()
            .flex_col()
            .justify_between()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        // Header Card (bg2)
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(px(10.0))
                            .rounded(px(8.0))
                            .bg(rgb(arclate::BG2_DARK))
                            .border_1()
                            .border_color(rgb(arclate::BORDER_CARD_DARK))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        img("app-logo.png")
                                            .size(px(36.0))
                                            .rounded(px(7.0)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(1.0))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(6.0))
                                                    .child(
                                                        div()
                                                            .text_size(px(15.0))
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                                                            .child("Winsentials"),
                                                    )
                                                    .child(
                                                        div()
                                                            .px(px(5.0))
                                                            .py(px(1.0))
                                                            .rounded(px(4.0))
                                                            .bg(rgba(0x70A2_D726))
                                                            .border_1()
                                                            .border_color(rgba(0x70A2_D74D))
                                                            .text_size(px(9.5))
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(rgb(arclate::BLUE_DARK))
                                                            .child(format!("v{APP_VERSION}")),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .text_color(rgb(arclate::TEXT_MUTED_DARK))
                                                    .child("Умная оптимизация, телеметрия и твики Windows"),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        // Destination Directory Picker (bg2 card, bg3 input)
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .p(px(8.0))
                            .rounded(px(8.0))
                            .bg(rgb(arclate::BG2_DARK))
                            .border_1()
                            .border_color(rgb(arclate::BORDER_CARD_DARK))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(arclate::TEXT_MUTED_DARK))
                                    .child("Папка установки"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .h(px(30.0))
                                            .px(px(8.0))
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .bg(rgb(arclate::BG3_DARK))
                                            .border_1()
                                            .border_color(rgb(arclate::BORDER_INPUT_DARK))
                                            .rounded(px(5.0))
                                            .child(
                                                Icon::new("icons/folder.svg")
                                                    .size(px(13.0))
                                                    .color(rgb(arclate::TEXT_MUTED_DARK)),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                                                    .overflow_hidden()
                                                    .child(dir_str),
                                            ),
                                    )
                                    .child(self.render_button(
                                        "browse_btn",
                                        "Обзор...",
                                        false,
                                        cx.listener(|this, _ev, _window, cx| {
                                            this.select_directory(cx);
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        // Options Card (bg2)
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .p(px(8.0))
                            .rounded(px(8.0))
                            .bg(rgb(arclate::BG2_DARK))
                            .border_1()
                            .border_color(rgb(arclate::BORDER_CARD_DARK))
                            .child({
                                let on_change = cx.listener(|this, &val: &bool, _window, cx| {
                                    this.desktop_shortcut = val;
                                    cx.notify();
                                });
                                self.render_switch_row(
                                    "icons/monitor.svg",
                                    "Ярлык на рабочем столе",
                                    "Добавить иконку быстрого запуска на рабочий стол",
                                    "opt-desktop",
                                    self.desktop_shortcut,
                                    move |val, window, cx| on_change(&val, window, cx),
                                )
                            })
                            .child(
                                div()
                                    .h(px(1.0))
                                    .bg(rgb(arclate::BORDER_CARD_DARK)),
                            )
                            .child({
                                let on_change = cx.listener(|this, &val: &bool, _window, cx| {
                                    this.start_menu_shortcut = val;
                                    cx.notify();
                                });
                                self.render_switch_row(
                                    "icons/layout-grid.svg",
                                    "Ярлык в меню «Пуск»",
                                    "Зарегистрировать в списке установленных программ",
                                    "opt-startmenu",
                                    self.start_menu_shortcut,
                                    move |val, window, cx| on_change(&val, window, cx),
                                )
                            })
                            .child(
                                div()
                                    .h(px(1.0))
                                    .bg(rgb(arclate::BORDER_CARD_DARK)),
                            )
                            .child({
                                let on_change = cx.listener(|this, &val: &bool, _window, cx| {
                                    this.launch_after = val;
                                    cx.notify();
                                });
                                self.render_switch_row(
                                    "icons/play.svg",
                                    "Запустить сразу после установки",
                                    "Открыть главное окно приложения по завершении",
                                    "opt-launch",
                                    self.launch_after,
                                    move |val, window, cx| on_change(&val, window, cx),
                                )
                            }),
                    ),
            )
            .child(
                // Bottom Button Bar
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.0))
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(arclate::BORDER_MAIN_DARK))
                    .child(self.render_button(
                        "cancel_btn",
                        "Отмена",
                        false,
                        |_ev, _window, cx| {
                            cx.quit();
                        },
                    ))
                    .child(self.render_button(
                        "install_btn",
                        "Установить",
                        true,
                        cx.listener(|this, _ev, _window, cx| {
                            this.start_installation(cx);
                        }),
                    )),
            )
    }

    fn render_switch_row(
        &self,
        icon_path: &'static str,
        title: &'static str,
        desc: &'static str,
        switch_id: &'static str,
        checked: bool,
        on_toggle: impl Fn(bool, &mut gpui::Window, &mut App) + 'static,
    ) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(1.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        Icon::new(icon_path)
                            .size(px(15.0))
                            .color(rgb(arclate::BLUE_DARK)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(rgb(arclate::TEXT_MUTED_DARK))
                                    .child(desc),
                            ),
                    ),
            )
            .child(Switch::new(switch_id, checked).on_toggle(on_toggle))
    }

    fn render_button(
        &self,
        id: &'static str,
        label: &'static str,
        is_primary: bool,
        on_click: impl Fn(&MouseDownEvent, &mut gpui::Window, &mut App) + 'static,
    ) -> gpui::Stateful<Div> {
        let base_bg = if is_primary {
            rgb(arclate::BLUE_DARK)
        } else {
            rgb(arclate::BG2_DARK)
        };

        let text_col = if is_primary {
            rgb(arclate::BG_DARK)
        } else {
            rgb(arclate::TEXT_PRIMARY_DARK)
        };

        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .h(px(30.0))
            .px(px(14.0))
            .rounded(px(5.0))
            .bg(base_bg)
            .border_1()
            .border_color(if is_primary {
                rgb(arclate::BLUE_DARK)
            } else {
                rgb(arclate::BORDER_CARD_DARK)
            })
            .text_size(px(12.0))
            .font_weight(if is_primary {
                FontWeight::BOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(text_col)
            .cursor_pointer()
            .hover(move |s| {
                if is_primary {
                    s.bg(rgba(0x70A2_D7E6))
                } else {
                    s.bg(rgb(arclate::BG3_DARK))
                        .border_color(rgb(arclate::BLUE_DARK))
                }
            })
            .active(move |s| {
                if is_primary {
                    s.bg(rgba(0x70A2_D7FF))
                } else {
                    s.bg(rgb(arclate::BG_DARK))
                }
            })
            .on_mouse_down(MouseButton::Left, on_click)
            .child(label)
    }

    fn render_installing(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(48.0))
                    .rounded_full()
                    .bg(rgba(0x70A2_D71F))
                    .child(
                        Icon::new("icons/loader.svg")
                            .size(px(24.0))
                            .color(rgb(arclate::BLUE_DARK)),
                    ),
            )
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                    .child("Идет установка Winsentials..."),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(arclate::TEXT_MUTED_DARK))
                    .child(self.status_text.clone()),
            )
    }

    fn render_finished(&self, _cx: &mut Context<Self>) -> Div {
        let launch_after = self.launch_after;

        div()
            .flex()
            .flex_col()
            .justify_between()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt(px(24.0))
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(44.0))
                            .rounded_full()
                            .bg(rgba(0x70D7_9526))
                            .border_1()
                            .border_color(rgba(0x70D7_954D))
                            .child(
                                Icon::new("icons/check-circle.svg")
                                    .size(px(24.0))
                                    .color(rgb(arclate::GREEN_DARK)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                            .child("Установка завершена!"),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(arclate::TEXT_MUTED_DARK))
                            .child("Winsentials успешно установлен и готов к работе."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(arclate::BORDER_MAIN_DARK))
                    .child(self.render_button(
                        "finish_btn",
                        if launch_after {
                            "Запустить Winsentials"
                        } else {
                            "Готово"
                        },
                        true,
                        move |_ev, _window, cx| {
                            if launch_after {
                                let exe = std::env::var("ProgramFiles").map_or_else(
                                    |_| {
                                        PathBuf::from(
                                            r"C:\Program Files\Winsentials\Winsentials.exe",
                                        )
                                    },
                                    |pf| {
                                        PathBuf::from(pf)
                                            .join("Winsentials")
                                            .join("Winsentials.exe")
                                    },
                                );
                                if exe.exists() {
                                    let _ = std::process::Command::new(exe).spawn();
                                }
                            }
                            cx.quit();
                        },
                    )),
            )
    }

    fn render_error(&self, _cx: &mut Context<Self>) -> Div {
        let err_msg = self
            .error_text
            .clone()
            .unwrap_or_else(|| SharedString::from("Произошла неизвестная ошибка при установке."));

        div()
            .flex()
            .flex_col()
            .justify_between()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt(px(20.0))
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(44.0))
                            .rounded_full()
                            .bg(rgba(0xD770_7026))
                            .border_1()
                            .border_color(rgba(0xD770_704D))
                            .child(
                                Icon::new("icons/alert-triangle.svg")
                                    .size(px(24.0))
                                    .color(rgb(arclate::RED_DARK)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(arclate::RED_DARK))
                            .child("Ошибка установки"),
                    )
                    .child(
                        div()
                            .max_w(px(420.0))
                            .text_size(px(11.5))
                            .text_color(rgb(arclate::TEXT_MUTED_DARK))
                            .child(err_msg),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(arclate::BORDER_MAIN_DARK))
                    .child(self.render_button(
                        "error_close_btn",
                        "Закрыть",
                        false,
                        |_ev, _window, cx| {
                            cx.quit();
                        },
                    )),
            )
    }

    fn render_uninstall_confirm(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .justify_between()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt(px(20.0))
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(44.0))
                            .rounded_full()
                            .bg(rgba(0xD770_701F))
                            .border_1()
                            .border_color(rgba(0xD770_704D))
                            .child(
                                Icon::new("icons/trash-2.svg")
                                    .size(px(22.0))
                                    .color(rgb(arclate::RED_DARK)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                            .child("Удаление Winsentials"),
                    )
                    .child(
                        div()
                            .max_w(px(420.0))
                            .text_size(px(11.5))
                            .text_color(rgb(arclate::TEXT_MUTED_DARK))
                            .child("Вы действительно хотите удалить Winsentials и все связанные ярлыки?"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.0))
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(arclate::BORDER_MAIN_DARK))
                    .child(self.render_button(
                        "uninst_cancel_btn",
                        "Отмена",
                        false,
                        |_ev, _window, cx| {
                            cx.quit();
                        },
                    ))
                    .child({
                        let on_click = cx.listener(|this, _ev, _window, cx| {
                            this.start_uninstallation(cx);
                        });
                        div()
                            .id("uninst_confirm_btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .h(px(30.0))
                            .px(px(14.0))
                            .rounded(px(5.0))
                            .bg(rgb(arclate::RED_DARK))
                            .border_1()
                            .border_color(rgb(arclate::RED_DARK))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xD770_70D9)))
                            .active(|s| s.bg(rgb(arclate::RED_DARK)))
                            .on_mouse_down(MouseButton::Left, on_click)
                            .child("Удалить")
                    }),
            )
    }

    fn render_uninstalling(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(48.0))
                    .rounded_full()
                    .bg(rgba(0xD770_701F))
                    .child(
                        Icon::new("icons/loader.svg")
                            .size(px(24.0))
                            .color(rgb(arclate::RED_DARK)),
                    ),
            )
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                    .child("Идет удаление Winsentials..."),
            )
    }

    fn render_uninstalled(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .justify_between()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .pt(px(24.0))
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(44.0))
                            .rounded_full()
                            .bg(rgba(0x70A2_D726))
                            .border_1()
                            .border_color(rgba(0x70A2_D74D))
                            .child(
                                Icon::new("icons/check-circle.svg")
                                    .size(px(24.0))
                                    .color(rgb(arclate::BLUE_DARK)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(arclate::TEXT_PRIMARY_DARK))
                            .child("Winsentials удален"),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(arclate::TEXT_MUTED_DARK))
                            .child("Программа была успешно удалена с вашего компьютера."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(arclate::BORDER_MAIN_DARK))
                    .child(self.render_button(
                        "uninst_close_btn",
                        "Закрыть",
                        false,
                        |_ev, _window, cx| {
                            cx.quit();
                        },
                    )),
            )
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_uninstall = args.iter().any(|a| a == "--uninstall" || a == "-u");

    Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(EmbeddedAssetSource)
        .run(move |cx: &mut App| {
            let fonts = vec![
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Regular.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Medium.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-SemiBold.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Bold.ttf")
                        .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../../assets/fonts/IBM Plex Mono/IBMPlexMono-Regular.ttf")
                        .as_slice(),
                ),
            ];

            cx.text_system().add_fonts(fonts).ok();

            cx.set_global(Theme::dark());

            let window_bounds =
                WindowBounds::Windowed(Bounds::centered(None, size(px(540.0), px(480.0)), cx));

            let window_options = WindowOptions {
                window_bounds: Some(window_bounds),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from(if is_uninstall {
                        "Удаление Winsentials"
                    } else {
                        "Установка Winsentials"
                    })),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(12.0), px(12.0))),
                }),
                window_background: WindowBackgroundAppearance::Opaque,
                window_min_size: Some(size(px(540.0), px(480.0))),
                focus: true,
                show: true,
                kind: gpui::WindowKind::PopUp,
                is_movable: true,
                display_id: None,
                ..Default::default()
            };

            let _ = cx.open_window(window_options, |_window, cx| {
                cx.new(|_cx| SetupView::new(is_uninstall))
            });
        });
}
