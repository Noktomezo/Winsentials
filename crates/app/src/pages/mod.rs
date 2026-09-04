pub mod cleanup_page;
pub mod context_menu_page;
pub mod cpu_page;
pub mod dashboard_page;
pub mod disk_page;
pub mod explorer_page;
pub mod gpu_page;
pub mod input_page;
pub mod interface_page;
pub mod network_page;
pub mod network_tweaks_page;
pub mod page_header;
pub mod ram_page;
pub mod security_page;
pub mod settings_page;
pub mod startup_page;
pub mod tools_page;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[allow(unused_imports)]
pub use cleanup_page::CleanupPage;
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
pub use network_tweaks_page::NetworkTweaksPage;
#[allow(unused_imports)]
pub use page_header::PageHeader;
#[allow(unused_imports)]
pub use ram_page::RamPage;
#[allow(unused_imports)]
pub use security_page::SystemPage;
#[allow(unused_imports)]
pub use settings_page::SettingsPage;
#[allow(unused_imports)]
pub use startup_page::StartupPage;
#[allow(unused_imports)]
pub use tools_page::ToolsPage;

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
    hovered_telemetry_card: Option<SharedString>,
    current_locale: &'static str,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    opening_dropdown: Option<&'static str>,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    gpu_engine_slots: &HashMap<(usize, usize), &'static str>,
    minimize_to_tray: bool,
    autostart: bool,
    autostart_to_tray: bool,
    discord_rpc: crate::features::discord_rpc::DiscordRpcActivity,
    check_updates: bool,
    update_state: &crate::features::updater::UpdateState,
    startup_entries: &[crate::entities::startup::StartupEntry],
    startup_filter: Option<crate::entities::startup::StartupSource>,
    startup_search_query: &str,
    startup_search_focused: bool,
    startup_search_hovered: bool,
    startup_search_selection: Option<(usize, usize)>,
    startup_search_focus: &gpui::FocusHandle,
    startup_open_menu_id: Option<&str>,
    hovered_startup_card: Option<String>,
    cleanup_page: CleanupPage,
    on_navigate: impl Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static,
    on_hover_telemetry_card: impl Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static,
    on_toggle_tweak: impl Fn(&'static str, bool, &mut Window, &mut App) + 'static,
    on_change_keyboard_repeat: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_ctf_optimization: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_snapkey: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_palette: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_language: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_theme: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_change_transparency: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_minimize_to_tray: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_autostart: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_toggle_autostart_to_tray: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_change_discord_rpc: impl Fn(&str, &mut Window, &mut App) + 'static,
    on_toggle_check_updates: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_check_update: impl Fn(&mut Window, &mut App) + 'static,
    on_download_and_install_update: impl Fn(&mut Window, &mut App) + 'static,
    on_select_gpu_engine: impl Fn(usize, usize, &'static str, &mut Window, &mut App) + 'static,
    on_reset_gpu_slots: impl Fn(usize, &mut Window, &mut App) + 'static,
    on_toggle_dropdown: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    on_hover_dropdown: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    on_hover_option: impl Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static,
    on_close_dropdowns: impl Fn(&mut Window, &mut App) + 'static,
    on_hover_tooltip: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
    on_toggle_startup: impl Fn(&crate::entities::startup::StartupEntry, &mut Window, &mut App) + 'static,
    on_delete_startup: impl Fn(&crate::entities::startup::StartupEntry, &mut Window, &mut App) + 'static,
    on_open_startup_folder: impl Fn(&crate::entities::startup::StartupEntry, &mut Window, &mut App)
    + 'static,
    on_open_startup_source: impl Fn(&crate::entities::startup::StartupEntry, &mut Window, &mut App)
    + 'static,
    on_copy_startup_path: impl Fn(&crate::entities::startup::StartupEntry, &mut Window, &mut App)
    + 'static,
    on_toggle_startup_menu: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
    on_select_startup_filter: impl Fn(
        Option<crate::entities::startup::StartupSource>,
        &mut Window,
        &mut App,
    ) + 'static,
    on_change_startup_search: impl Fn(String, &mut Window, &mut App) + 'static,
    on_hover_startup_search: impl Fn(&bool, &mut Window, &mut App) + 'static,
    on_focus_startup_search: impl Fn(bool, &mut Window, &mut App) + 'static,
    on_selection_startup_search: impl Fn(Option<(usize, usize)>, &mut Window, &mut App) + 'static,
    on_hover_startup_card: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let on_nav_arc = Arc::new(on_navigate);
    let on_nav_dash = on_nav_arc.clone();
    let on_nav_cpu = on_nav_arc.clone();
    let on_nav_tools = on_nav_arc;

    let on_hover_card_arc = Arc::new(on_hover_telemetry_card);
    let on_hover_card_dash = on_hover_card_arc.clone();
    let on_hover_card_tools = on_hover_card_arc;

    let on_select_gpu_engine_arc = Arc::new(on_select_gpu_engine);
    let on_reset_gpu_slots_arc = Arc::new(on_reset_gpu_slots);

    let on_toggle_tweak_arc = Arc::new(on_toggle_tweak);
    let on_toggle_tweak_ctx = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_exp = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_iface = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_input = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_security = on_toggle_tweak_arc.clone();
    let on_toggle_tweak_net = on_toggle_tweak_arc;

    let on_toggle_dropdown_arc = Arc::new(on_toggle_dropdown);
    let on_toggle_dd_set = on_toggle_dropdown_arc.clone();
    let on_toggle_dd_gpu = on_toggle_dropdown_arc.clone();
    let on_toggle_dd_input = on_toggle_dropdown_arc;

    let on_hover_dropdown_arc = Arc::new(on_hover_dropdown);
    let on_hover_dd_set = on_hover_dropdown_arc.clone();
    let on_hover_dd_gpu = on_hover_dropdown_arc.clone();
    let on_hover_dd_input = on_hover_dropdown_arc;

    let on_hover_option_arc = Arc::new(on_hover_option);
    let on_hover_opt_set = on_hover_option_arc.clone();
    let on_hover_opt_gpu = on_hover_option_arc.clone();
    let on_hover_opt_input = on_hover_option_arc;

    let on_close_dropdowns_arc = Arc::new(on_close_dropdowns);
    let on_close_dd_set = on_close_dropdowns_arc.clone();
    let on_close_dd_gpu = on_close_dropdowns_arc.clone();
    let on_close_dd_input = on_close_dropdowns_arc;

    let on_hover_tooltip_arc = Arc::new(on_hover_tooltip);
    let on_hover_tt_ctx = on_hover_tooltip_arc.clone();
    let on_hover_tt_exp = on_hover_tooltip_arc.clone();
    let on_hover_tt_iface = on_hover_tooltip_arc.clone();
    let on_hover_tt_input = on_hover_tooltip_arc.clone();
    let on_hover_tt_security = on_hover_tooltip_arc.clone();
    let on_hover_tt_net = on_hover_tooltip_arc.clone();
    let on_hover_tt_startup = on_hover_tooltip_arc.clone();
    let on_hover_tt_settings = on_hover_tooltip_arc;

    let page_element = match route {
        AppRoute::Dashboard => DashboardPage::new(telemetry, hovered_telemetry_card.clone())
            .on_hover_card(move |id, val, window, cx| {
                on_hover_card_dash(id, val, window, cx);
            })
            .on_navigate(move |target_route, window, cx| {
                on_nav_dash(target_route, window, cx);
            })
            .into_any_element(),
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
                        opening_dropdown,
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
        AppRoute::ContextMenu => ContextMenuPage::new(windows_build)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_ctx(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_ctx(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Explorer => ExplorerPage::new(windows_build)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_exp(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_exp(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Interface => InterfacePage::new(windows_build)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_iface(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_iface(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Input => InputPage::new(
            windows_build,
            open_dropdown,
            open_dropdown_upward,
            opening_dropdown,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            pending_selection,
        )
        .on_toggle_tweak(move |id, val, window, cx| {
            on_toggle_tweak_input(id, val, window, cx);
        })
        .on_select_preset(on_change_keyboard_repeat)
        .on_select_ctf_preset(on_change_ctf_optimization)
        .on_select_snapkey_preset(on_change_snapkey)
        .on_toggle_dropdown(move |id, window, cx| {
            on_toggle_dd_input(id, window, cx);
        })
        .on_hover_dropdown(move |id, hovered, window, cx| {
            on_hover_dd_input(id, hovered, window, cx);
        })
        .on_hover_option(move |id, option, hovered, window, cx| {
            on_hover_opt_input(id, option, hovered, window, cx);
        })
        .on_close_dropdowns(move |window, cx| {
            on_close_dd_input(window, cx);
        })
        .on_hover_tooltip(move |tt, window, cx| {
            on_hover_tt_input(tt, window, cx);
        })
        .into_any_element(),
        AppRoute::System => SystemPage::new(windows_build)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_security(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_security(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::NetworkTweaks => NetworkTweaksPage::new(windows_build)
            .on_toggle_tweak(move |id, val, window, cx| {
                on_toggle_tweak_net(id, val, window, cx);
            })
            .on_hover_tooltip(move |tt, window, cx| {
                on_hover_tt_net(tt, window, cx);
            })
            .into_any_element(),
        AppRoute::Tools => ToolsPage::new(hovered_telemetry_card)
            .on_hover_card(move |id, val, window, cx| {
                on_hover_card_tools(id, val, window, cx);
            })
            .on_navigate(move |r, window, cx| {
                on_nav_tools(r, window, cx);
            })
            .into_any_element(),
        AppRoute::Startup => StartupPage::new(
            startup_entries.to_vec(),
            startup_filter,
            startup_search_query,
            startup_search_focused,
            startup_search_hovered,
            startup_search_selection,
            startup_open_menu_id.map(ToString::to_string),
            hovered_startup_card,
        )
        .search_focus(startup_search_focus)
        .on_change_search(on_change_startup_search)
        .on_hover_search(on_hover_startup_search)
        .on_focus_search(on_focus_startup_search)
        .on_selection_search(on_selection_startup_search)
        .on_hover_card(on_hover_startup_card)
        .on_toggle(on_toggle_startup)
        .on_delete(on_delete_startup)
        .on_open_folder(on_open_startup_folder)
        .on_open_source(on_open_startup_source)
        .on_copy_path(on_copy_startup_path)
        .on_hover_tooltip(move |tt, window, cx| {
            on_hover_tt_startup(tt, window, cx);
        })
        .on_toggle_menu(on_toggle_startup_menu)
        .on_select_filter(on_select_startup_filter)
        .into_any_element(),
        AppRoute::Cleanup => cleanup_page.into_any_element(),
        AppRoute::Settings => SettingsPage::new(
            current_locale,
            minimize_to_tray,
            autostart,
            autostart_to_tray,
            discord_rpc,
            check_updates,
            update_state.clone(),
            open_dropdown,
            open_dropdown_upward,
            opening_dropdown,
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
        .on_toggle_check_updates(on_toggle_check_updates)
        .on_check_update(on_check_update)
        .on_download_and_install_update(on_download_and_install_update)
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
        .on_hover_tooltip(move |tt, window, cx| {
            on_hover_tt_settings(tt, window, cx);
        })
        .into_any_element(),
    };
    let anim_id = format!("page_enter_{}", route.id());

    let page_container = if route == AppRoute::Startup {
        page_element
    } else {
        SmoothScroll::new(route.id(), page_element).into_any_element()
    };

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
        .child(page_container)
        .into_any_element()
}
