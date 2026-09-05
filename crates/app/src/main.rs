#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, point, px, size,
};
use std::rc::Rc;

use winsentials::app::AppView;
use winsentials::features;
use winsentials::shared::assets::EmbeddedAssetSource;
use winsentials::shared::theme::Theme;

fn main() {
    let Some(_single_instance) = features::single_instance::try_acquire_single_instance() else {
        return;
    };

    rust_i18n::set_locale("ru");

    let platform =
        gpui_windows::WindowsPlatform::new(false).expect("failed to initialize Windows platform");

    Application::with_platform(Rc::new(platform))
        .with_assets(EmbeddedAssetSource)
        .run(|cx: &mut App| {
            let fonts = vec![
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Regular.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Medium.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-SemiBold.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!(
                        "../../../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Bold.ttf"
                    )
                    .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../../../assets/fonts/IBM Plex Mono/IBMPlexMono-Regular.ttf")
                        .as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../../../assets/fonts/IBM Plex Mono/IBMPlexMono-Medium.ttf")
                        .as_slice(),
                ),
            ];
            cx.text_system().add_fonts(fonts).ok();

            cx.set_global(Theme::dark());
            cx.set_global(winsentials::entities::tweaks::TweakStates::load_initial());

            let start_in_tray = std::env::args().any(|arg| arg == "--tray" || arg == "--minimized");

            let app_title = if cfg!(debug_assertions) {
                "Winsentials (Dev)"
            } else {
                "Winsentials"
            };

            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(100.0), px(100.0)),
                    size: size(px(900.0), px(700.0)),
                })),
                window_min_size: Some(size(px(900.0), px(700.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some(app_title.into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                window_background: WindowBackgroundAppearance::Blurred,
                is_resizable: false,
                focus: !start_in_tray,
                show: !start_in_tray,
                ..Default::default()
            };

            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| {
                    let mut view = AppView::new();
                    view.start_telemetry_polling(cx);
                    view.start_tray_listener(cx);
                    view.start_updater_polling(cx);
                    view
                });

                let view_clone = view.downgrade();
                window.on_window_should_close(cx, move |_window, cx| {
                    if let Some(view) = view_clone.upgrade() {
                        let minimize_to_tray = view.read(cx).minimize_to_tray();
                        if minimize_to_tray {
                            winsentials::features::tray::hide_main_window();
                            false
                        } else {
                            cx.quit();
                            true
                        }
                    } else {
                        true
                    }
                });

                view
            })
            .expect("failed to open window");
        });
}
