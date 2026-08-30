pub mod context_menu_page;
pub mod cpu_page;
pub mod dashboard_page;
pub mod disk_page;
pub mod explorer_page;
pub mod gpu_page;
pub mod input_page;
pub mod interface_page;
pub mod network_page;
pub mod page_header;
pub mod ram_page;
pub mod settings_page;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[allow(unused_imports)]
pub use context_menu_page::ContextMenuPage;
#[allow(unused_imports)]
pub use cpu_page::CpuPage;
#[allow(unused_imports)]
pub use dashboard_page::DashboardPage;
#[allow(unused_imports)]
pub use disk_page::DiskPage;
#[allow(unused_imports)]
pub use explorer_page::ExplorerPage;
#[allow(unused_imports)]
pub use gpu_page::GpuPage;
#[allow(unused_imports)]
pub use input_page::InputPage;
#[allow(unused_imports)]
pub use interface_page::InterfacePage;
#[allow(unused_imports)]
pub use network_page::NetworkPage;
#[allow(unused_imports)]
pub use page_header::PageHeader;
#[allow(unused_imports)]
pub use ram_page::RamPage;
#[allow(unused_imports)]
pub use settings_page::SettingsPage;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, IntoElement, ParentElement, SharedString,
    Styled, Window, div, ease_in_out, px,
};

use crate::entities::TelemetryData;
use crate::features::navigation::AppRoute;
use crate::shared::ui::{SmoothScroll, TooltipState};

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::fn_params_excessive_bools
)]
#[must_use]
pub fn render_route(
    route: AppRoute,
    telemetry: TelemetryData,
    windows_build: u32,
    sidebar_expanded: bool,
    hovered_telemetry_card: Option<SharedString>,
    current_locale: &'static str,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    gpu_engine_slots: &HashMap<(usize, usize), &'static str>,
    minimize_to_tray: bool,
    autostart: bool,
    autostart_to_tray: bool,
    discord_rpc: crate::features::discord_rpc::DiscordRpcActivity,
    on_navigate: impl Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static,
    on_hover_telemetry_card: impl Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static,
    on_toggle_tweak: impl Fn(&'static str, bool, &mut Window, &mut App) + 'static,
    on_change_palette: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_language: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_theme: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_transparency: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_minimize_to_tray: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_autostart: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_autostart_to_tray: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_change_discord_rpc: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_select_gpu_engine: impl Fn(usize, usize, &'static str, &mut Window, &mut App) + 'static,
    on_reset_gpu_slots: impl Fn(usize, &mut Window, &mut App) + 'static,
    on_toggle_dropdown: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    on_hover_dropdown: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    on_hover_option: impl Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static,
    on_close_dropdowns: impl Fn(&mut Window, &mut App) + 'static,
    on_hover_tooltip: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let on_nav_arc = Arc::new(on_navigate);
    let on_nav_dash = on_nav_arc.clone();
    let on_nav_cpu = on_nav_arc;

    let on_select_gpu_engine_arc = Arc::new(on_select_gpu_engine);
    let on_reset_gpu_slots_arc = Arc::new(on_reset_gpu_slots);

    let on_toggle_tweak_arc = Arc::new(on_toggle_tweak);
    let on_toggle_tweak_ctx = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_exp = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_iface = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_input = on_toggle_tweak_arc;

    let on_toggle_dropdown_arc = Arc::new(on_toggle_dropdown);
    let on_toggle_dd_set = on_toggle_dropdown_arc.clone();
    let on_toggle_dd_gpu = on_toggle_dropdown_arc;

    let on_hover_dropdown_arc = Arc::new(on_hover_dropdown);
    let on_hover_dd_set = on_hover_dropdown_arc.clone();
    let on_hover_dd_gpu = on_hover_dropdown_arc;

    let on_hover_option_arc = Arc::new(on_hover_option);
    let on_hover_opt_set = on_hover_option_arc.clone();
    let on_hover_opt_gpu = on_hover_option_arc;

    let on_close_dropdowns_arc = Arc::new(on_close_dropdowns);
    let on_close_dd_set = on_close_dropdowns_arc.clone();
    let on_close_dd_gpu = on_close_dropdowns_arc;

    let on_hover_tooltip_arc = Arc::new(on_hover_tooltip);
    let on_hover_tt_ctx = on_hover_tooltip_arc.clone();
    let on_hover_tt_exp = on_hover_tooltip_arc.clone();
    let on_hover_tt_iface = on_hover_tooltip_arc.clone();
    let on_hover_tt_input = on_hover_tooltip_arc;

    let page_element = match route {
        AppRoute::Dashboard => {
            DashboardPage::new(telemetry, hovered_telemetry_card, sidebar_expanded)
                .on_hover_card(on_hover_telemetry_card)
                .on_navigate(move |target_route, window, cx| {
                    on_nav_dash(target_route, window, cx);
                })
                .into_any_element()
        }
        AppRoute::CpuDetail => CpuPage::new(telemetry.cpu_detail)
            .on_navigate(move |target_route, window, cx| {
                on_nav_cpu(target_route, window, cx);
            })
            .into_any_element(),
        AppRoute::RamDetail => RamPage::new(telemetry.ram).into_any_element(),
        AppRoute::DiskDetail(id) => telemetry
            .disks
            .into_iter()
            .find(|disk| disk.id == id)
            .map_or_else(
                || {
                    div()
                        .p(px(16.0))
                        .child(rust_i18n::t!("disk_detail.unavailable"))
                        .into_any_element()
                },
                |disk| DiskPage::new(disk).into_any_element(),
            ),
        AppRoute::NetworkDetail(id) => telemetry
            .networks
            .into_iter()
            .find(|network| network.id == id)
            .map_or_else(
                || {
                    div()
                        .p(px(16.0))
                        .child(rust_i18n::t!("network_detail.unavailable"))
                        .into_any_element()
                },
                |network| NetworkPage::new(network).into_any_element(),
            ),
        AppRoute::GpuDetail(id) => telemetry
            .gpus
            .into_iter()
            .find(|gpu| gpu.id == id)
            .map_or_else(
                || {
                    div()
                        .p(px(16.0))
                        .child(rust_i18n::t!("gpu_detail.unavailable"))
                        .into_any_element()
                },
                |gpu| {
                    let default_slots: [&'static str; 4] = if gpu.is_discrete {
                        ["3D", "Copy", "Video Encode", "Video Decode"]
                    } else {
                        ["3D", "Copy", "High Priority Compute", "High Priority 3D"]
                    };
                    let mut slots = default_slots;
                    for (i, slot) in slots.iter_mut().enumerate() {
                        if let Some(&engine) = gpu_engine_slots.get(&(gpu.id, i)) {
                            *slot = engine;
                        }
                    }

                    let on_select_eng = on_select_gpu_engine_arc.clone();
                    let on_reset_sl = on_reset_gpu_slots_arc.clone();
                    let on_toggle_dd = on_toggle_dd_gpu.clone();
                    let on_hover_dd = on_hover_dd_gpu.clone();
                    let on_hover_opt = on_hover_opt_gpu.clone();
                    let on_close_dd = on_close_dd_gpu.clone();
                    let gpu_id = gpu.id;

                    GpuPage::new(
                        gpu,
                        slots,
                        open_dropdown,
                        closing_dropdown,
                        hovered_dropdown,
                        hovered_option,
                    )
                    .on_select_engine(move |slot_idx, eng, window, cx| {
                        on_select_eng(gpu_id, slot_idx, eng, window, cx);
                    })
                    .on_reset_slots(move |window, cx| {
                        on_reset_sl(gpu_id, window, cx);
                    })
                    .on_toggle_dropdown(move |d_id, window, cx| {
                        on_toggle_dd(d_id, window, cx);
                    })
                    .on_hover_dropdown(move |d_id, hov, window, cx| {
                        on_hover_dd(d_id, hov, window, cx);
                    })
                    .on_hover_option(move |d_id, opt, hov, window, cx| {
                        on_hover_opt(d_id, opt, hov, window, cx);
                    })
                    .on_close_dropdowns(move |window, cx| {
                        on_close_dd(window, cx);
                    })
                    .into_any_element()
                },
            ),
        AppRoute::ContextMenu => ContextMenuPage::new(windows_build, sidebar_expanded)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_ctx(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_ctx(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Explorer => ExplorerPage::new(windows_build, sidebar_expanded)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_exp(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_exp(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Interface => InterfacePage::new(windows_build, sidebar_expanded)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_iface(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_iface(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Input => InputPage::new(windows_build, sidebar_expanded)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_input(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_input(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Settings => SettingsPage::new(
            current_locale,
            minimize_to_tray,
            autostart,
            autostart_to_tray,
            discord_rpc,
            open_dropdown,
            open_dropdown_upward,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            pending_selection,
        )
        .on_change_palette(on_change_palette)
        .on_change_language(on_change_language)
        .on_change_theme(on_change_theme)
        .on_change_transparency(on_change_transparency)
        .on_toggle_minimize_to_tray(on_toggle_minimize_to_tray)
        .on_toggle_autostart(on_toggle_autostart)
        .on_toggle_autostart_to_tray(on_toggle_autostart_to_tray)
        .on_change_discord_rpc(on_change_discord_rpc)
        .on_toggle_dropdown(move |id, window, cx| {
            on_toggle_dd_set(id, window, cx);
        })
        .on_hover_dropdown(move |id, hov, window, cx| {
            on_hover_dd_set(id, hov, window, cx);
        })
        .on_hover_option(move |d_id, opt, hov, window, cx| {
            on_hover_opt_set(d_id, opt, hov, window, cx);
        })
        .on_close_dropdowns(move |window, cx| {
            on_close_dd_set(window, cx);
        })
        .into_any_element(),
    };

    let anim_id = format!("page_enter_{}", route.id());

    div()
        .size_full()
        .with_animation(
            ElementId::Name(anim_id.into()),
            Animation::new(Duration::from_millis(180)).with_easing(ease_in_out),
            move |page_box, delta| {
                let opacity = delta;
                let offset_y = (1.0 - delta) * 10.0;
                page_box.opacity(opacity).mt(px(offset_y))
            },
        )
        .child(SmoothScroll::new(route.id(), page_element))
        .into_any_element()
}
