use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Transformation, Window, div, point, pulsating_between, px,
    svg,
};
use rust_i18n::t;

use crate::entities::{SystemInfo, TelemetryData};
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::lerp_rgba;

pub type TelemetryCardHoverHandler =
    Arc<dyn Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static>;
pub type DashboardNavigateHandler =
    Arc<dyn Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static>;
pub type TelemetryCardClickHandler = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync + 'static>;

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

fn render_info_row(
    label: impl Into<SharedString>,
    value_el: impl IntoElement,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .gap(px(8.0))
        .py(px(2.5))
        .child(
            div()
                .flex_none()
                .text_size(px(12.0))
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .justify_end()
                .min_w(px(0.0))
                .child(value_el),
        )
}

fn render_text_val(text: impl Into<SharedString>, theme: &Theme) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_primary)
        .text_ellipsis()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(text.into())
}

fn render_activation_val(is_activated: bool, theme: &Theme) -> impl IntoElement {
    let (color, text_key) = if is_activated {
        (theme.accent_green, "system.activated")
    } else {
        (theme.accent_red, "system.not_activated")
    };

    // Smooth organic breathing pulsation between 40% and 100% opacity
    let pulse_animation = Animation::new(Duration::from_millis(2200))
        .repeat()
        .with_easing(pulsating_between(0.4, 1.0));

    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(t!(text_key))
        .with_animation(
            ElementId::Name("activation_pulse".into()),
            pulse_animation,
            gpui::Styled::opacity,
        )
}

fn semantic_percent_color(pct: f32, theme: &Theme) -> Rgba {
    if pct > 85.0 {
        theme.accent_red
    } else if pct >= 60.0 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

fn semantic_gpu_color(usage_pct: u32, temp_c: u32, theme: &Theme) -> Rgba {
    if usage_pct > 85 || temp_c > 80 {
        theme.accent_red
    } else if usage_pct >= 60 || temp_c >= 65 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

fn render_metric_label(text: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .text_ellipsis()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(text.into())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_telemetry_card(
    card_id: &'static str,
    icon: impl Into<SharedString>,
    icon_color: Rgba,
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    metric_el: impl IntoElement,
    is_hovered: bool,
    theme: &Theme,
    on_hover: Option<TelemetryCardHoverHandler>,
    on_click: Option<TelemetryCardClickHandler>,
) -> impl IntoElement {
    let target = if is_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(320.0, 26.0, 1.0))
        .to(target)
        .with_epsilon(0.005);

    let icon_box = div()
        .size(px(32.0))
        .rounded(px(6.0))
        .bg(theme.input_bg)
        .border_1()
        .border_color(theme.card_border)
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .child(Icon::new(icon).size(px(16.0)).color(icon_color));

    let text_stack = div()
        .flex()
        .flex_col()
        .justify_between()
        .h(px(32.0))
        .flex_1()
        .min_w(px(0.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .min_w(px(0.0))
                .child(
                    div()
                        .flex_none()
                        .text_size(px(13.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .whitespace_nowrap()
                        .child(title.into()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.5))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_muted)
                        .text_ellipsis()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(detail.into()),
                ),
        )
        .child(div().flex().items_center().min_w(px(0.0)).child(metric_el));

    let text_muted = theme.text_muted;
    let text_primary = theme.text_primary;

    let chevron = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .with_spring(
            ElementId::Name(format!("{card_id}_chev_spring").into()),
            spring.clone(),
            move |chev, val| {
                let t = val.clamp(0.0, 1.0);
                let slide_x = t * 5.0;
                let col = lerp_rgba(text_muted, text_primary, t);
                chev.child(
                    svg()
                        .path("icons/chevron-right.svg")
                        .size(px(14.0))
                        .text_color(col)
                        .with_transformation(Transformation::translate(point(px(slide_x), px(0.0))))
                        .flex_none(),
                )
            },
        );

    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let input_border = theme.input_border;

    let on_hov = on_hover;
    let id_str: SharedString = card_id.into();

    let mut card_el = div()
        .id(ElementId::Name(format!("{card_id}_root").into()))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(10.0))
        .rounded(px(10.0))
        .border_1()
        .p(px(16.0))
        .h(px(64.0))
        .w_full()
        .on_hover(move |&hovered, window, cx| {
            if let Some(ref h) = on_hov {
                h(id_str.clone(), hovered, window, cx);
            }
        });

    if let Some(on_clk) = on_click {
        card_el = card_el.on_click(move |_, window, cx| {
            on_clk(window, cx);
        });
    }

    card_el
        .with_spring(
            ElementId::Name(format!("{card_id}_bg_spring").into()),
            spring,
            move |card, val| {
                let t = val.clamp(0.0, 1.0);
                let bg = lerp_rgba(card_bg, input_bg, t);
                let border = lerp_rgba(card_border, input_border, t);
                card.bg(bg).border_color(border)
            },
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .flex_1()
                .min_w(px(0.0))
                .child(icon_box)
                .child(text_stack),
        )
        .child(chevron)
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

        // 1. Top System Information Card (Static, no right chevron)
        let left_col = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("system.os_version"),
                render_text_val(info.os_version, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.motherboard"),
                render_text_val(info.motherboard, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.user"),
                render_text_val(info.username, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.architecture"),
                render_text_val(info.architecture, &theme),
                &theme,
            ));

        let right_col = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("system.build"),
                render_text_val(info.build_number, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.computer_name"),
                render_text_val(info.computer_name, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.tweaks_applied"),
                render_text_val(info.tweaks_applied, &theme),
                &theme,
            ))
            .child(render_info_row(
                t!("system.activation"),
                render_activation_val(info.is_activated, &theme),
                &theme,
            ));

        let grid_row = div()
            .flex()
            .flex_row()
            .gap(px(16.0))
            .w_full()
            .min_w(px(0.0))
            .child(left_col)
            .child(right_col);

        let system_card =
            GroupCard::new("icons/monitor.svg", t!("system.title"), t!("system.desc"))
                .icon_color(theme.accent_blue)
                .child(grid_row);

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
