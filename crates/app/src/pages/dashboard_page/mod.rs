use std::sync::Arc;

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
};
use rust_i18n::t;

use crate::entities::{SystemInfo, TelemetryData};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub mod system_card;
pub mod telemetry_card;

pub(crate) use system_card::*;
pub use telemetry_card::*;

pub type DashboardNavigateHandler =
    Arc<dyn Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static>;
#[derive(IntoElement)]
pub struct DashboardPage {
    telemetry: TelemetryData,
    hovered_card: Option<SharedString>,
    on_hover_card: Option<TelemetryCardHoverHandler>,
    on_navigate: Option<DashboardNavigateHandler>,
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new(TelemetryData::fetch(), None)
    }
}

impl DashboardPage {
    #[must_use]
    pub fn new(telemetry: TelemetryData, hovered_card: Option<SharedString>) -> Self {
        Self {
            telemetry,
            hovered_card,
            on_hover_card: None,
            on_navigate: None,
        }
    }

    #[must_use]
    pub fn on_hover_card(
        mut self,
        handler: impl Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_hover_card = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_navigate(
        mut self,
        handler: impl Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_navigate = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for DashboardPage {
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Dashboard;
        let info = SystemInfo::fetch();
        let telemetry = self.telemetry;
        let hovered_card = self.hovered_card;
        let on_hover_card = self.on_hover_card;
        let on_navigate = self.on_navigate;

        let system_card = render_system_card(&info, &theme);

        // 2. Interactive telemetry cards in a fixed two-column grid.
        let mut card_items: Vec<(&'static str, AnyElement)> = Vec::new();
        // CPU Card: Uniform Blue accent icon - Navigates to CpuDetail
        let on_nav_cpu = on_navigate.clone();
        let cpu_color = semantic_percent_color(telemetry.cpu.usage_percent as f32, &theme);
        let cpu_hovered = hovered_card.as_deref() == Some("card_cpu");
        card_items.push((
            "card_cpu",
            render_telemetry_card(
                "card_cpu",
                "icons/cpu.svg",
                theme.accent_blue,
                t!("telemetry.cpu"),
                format!("({})", telemetry.cpu.name),
                render_metric_label(format!("{}%", telemetry.cpu.usage_percent), cpu_color),
                cpu_hovered,
                &theme,
                on_hover_card.clone(),
                Some(Arc::new(move |window, cx| {
                    if let Some(ref nav) = on_nav_cpu {
                        nav(AppRoute::CpuDetail, window, cx);
                    }
                })),
            )
            .into_any_element(),
        ));

        // RAM Card: Uniform Blue accent icon - Navigates to RamDetail
        let on_nav_ram = on_navigate.clone();
        let ram_pct = (telemetry.ram.used_gb / telemetry.ram.total_gb.max(1.0)) * 100.0;
        let ram_color = semantic_percent_color(ram_pct, &theme);
        let ram_hovered = hovered_card.as_deref() == Some("card_ram");
        card_items.push((
            "card_ram",
            render_telemetry_card(
                "card_ram",
                "icons/memory-stick.svg",
                theme.accent_blue,
                t!("telemetry.ram"),
                format!("({})", telemetry.ram.slots),
                render_metric_label(
                    format!(
                        "{:.1} / {:.1} {}",
                        telemetry.ram.used_gb,
                        telemetry.ram.total_gb,
                        t!("telemetry.gb")
                    ),
                    ram_color,
                ),
                ram_hovered,
                &theme,
                on_hover_card.clone(),
                Some(Arc::new(move |window, cx| {
                    if let Some(ref nav) = on_nav_ram {
                        nav(AppRoute::RamDetail, window, cx);
                    }
                })),
            )
            .into_any_element(),
        ));

        // Disk Cards: Uniform Blue accent icon
        for disk in telemetry.disks {
            let on_nav_disk = on_navigate.clone();
            let disk_route = AppRoute::DiskDetail(disk.id);
            let detail = if let Some(ref custom_name) = disk.custom_name {
                format!("({} - {})", disk.letter, custom_name)
            } else {
                format!("({})", disk.letter)
            };
            let disk_pct = (disk.used_gb as f32 / disk.total_gb.max(1) as f32) * 100.0;
            let disk_color = semantic_percent_color(disk_pct, &theme);
            let disk_id_static: &'static str =
                Box::leak(format!("card_disk_{}", disk.id).into_boxed_str());
            let disk_hovered = hovered_card.as_deref() == Some(disk_id_static);

            card_items.push((
                disk_id_static,
                render_telemetry_card(
                    disk_id_static,
                    "icons/hard-drive.svg",
                    theme.accent_blue,
                    format!("{} {}", t!("telemetry.disk"), disk.id),
                    detail,
                    render_metric_label(
                        format!(
                            "{} / {} {}",
                            disk.used_gb,
                            disk.total_gb,
                            t!("telemetry.gb")
                        ),
                        disk_color,
                    ),
                    disk_hovered,
                    &theme,
                    on_hover_card.clone(),
                    Some(Arc::new(move |window, cx| {
                        if let Some(ref nav) = on_nav_disk {
                            nav(disk_route, window, cx);
                        }
                    })),
                )
                .into_any_element(),
            ));
        }

        let network_count = telemetry.networks.len();
        for (index, network) in telemetry.networks.into_iter().enumerate() {
            let on_nav_network = on_navigate.clone();
            let network_route = AppRoute::NetworkDetail(network.id);
            let card_id: &'static str =
                Box::leak(format!("card_network_{}", network.id).into_boxed_str());
            let network_hovered = hovered_card.as_deref() == Some(card_id);
            let title = if network_count == 1 {
                t!("telemetry.network").to_string()
            } else {
                format!("{} {index}", t!("telemetry.network"))
            };
            let network_metric = div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .min_w(px(0.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.0))
                        .child(
                            Icon::new("icons/arrow-down.svg")
                                .size(px(12.0))
                                .color(theme.accent_blue),
                        )
                        .child(render_metric_label(network.rx_speed, theme.accent_blue)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.0))
                        .child(
                            Icon::new("icons/arrow-up.svg")
                                .size(px(12.0))
                                .color(theme.accent_cyan),
                        )
                        .child(render_metric_label(network.tx_speed, theme.accent_cyan)),
                );

            card_items.push((
                card_id,
                render_telemetry_card(
                    card_id,
                    "icons/network.svg",
                    theme.accent_blue,
                    title,
                    format!("({} · {})", network.interface_name, network.adapter_name),
                    network_metric,
                    network_hovered,
                    &theme,
                    on_hover_card.clone(),
                    Some(Arc::new(move |window, cx| {
                        if let Some(ref nav) = on_nav_network {
                            nav(network_route, window, cx);
                        }
                    })),
                )
                .into_any_element(),
            ));
        }

        // GPU Cards: Uniform Blue accent icon
        for gpu in telemetry.gpus {
            let gpu_color = semantic_gpu_color(gpu.usage_percent, gpu.temperature_c, &theme);
            let gpu_id_static: &'static str =
                Box::leak(format!("card_gpu_{}", gpu.id).into_boxed_str());
            let gpu_hovered = hovered_card.as_deref() == Some(gpu_id_static);
            let nav_handler = on_navigate.clone();
            let target_gpu_route = AppRoute::GpuDetail(gpu.id);

            card_items.push((
                gpu_id_static,
                render_telemetry_card(
                    gpu_id_static,
                    "icons/circuit-board.svg",
                    theme.accent_blue,
                    format!("{} {}", t!("telemetry.gpu"), gpu.id),
                    format!("({})", gpu.name),
                    render_metric_label(
                        format!("{}% ({}°C)", gpu.usage_percent, gpu.temperature_c),
                        gpu_color,
                    ),
                    gpu_hovered,
                    &theme,
                    on_hover_card.clone(),
                    Some(Arc::new(move |window, cx| {
                        if let Some(ref h) = nav_handler {
                            h(target_gpu_route, window, cx);
                        }
                    })),
                )
                .into_any_element(),
            ));
        }

        let telemetry_grid = div()
            .grid()
            .grid_cols(2)
            .gap(px(12.0))
            .children(card_items.into_iter().map(|(_, card)| card));

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
            .child(system_card)
            .child(telemetry_grid)
    }
}
