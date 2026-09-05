use gpui::{div, px, IntoElement, ParentElement, Styled};

use crate::features::navigation::AppRoute;
use crate::shared::motion::hover_spring;
use crate::shared::theme::Theme;
use crate::shared::ui::Chip;
use super::state::*;

pub(crate) fn render_fps_bottlenecks_inspector(
    current_route: AppRoute,
    snapshot: &DevPerfSnapshot,
    on_freeze_telemetry: &DevActionCallback,
    on_chart_anim: &DevActionCallback,
    on_continuous: &DevActionCallback,
    on_hover_control: &Option<HoverControlHandler>,
    theme: &Theme,
) -> impl IntoElement {
    let route_weight = match current_route {
        AppRoute::CpuDetail
        | AppRoute::RamDetail
        | AppRoute::DiskDetail(_)
        | AppRoute::NetworkDetail(_)
        | AppRoute::GpuDetail(_) => "High (Glide)",
        AppRoute::ContextMenu => "Medium (Cards)",
        AppRoute::Cleanup => "High (Scan)",
        _ => "Normal",
    };

    let on_freeze_telemetry = on_freeze_telemetry.clone();
    let on_chart_anim = on_chart_anim.clone();
    let on_continuous = on_continuous.clone();

                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .p(px(8.0))
                        .rounded(px(8.0))
                        .bg(theme.input_bg)
                        .border_1()
                        .border_color(theme.input_border)
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.accent_cyan)
                                .child("FPS Bottlenecks Inspector"),
                        )
                        // Route tree complexity
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .gap(px(8.0))
                                .text_xs()
                                .child(
                                    div()
                                        .flex_1()
                                        .truncate()
                                        .text_color(theme.text_muted)
                                        .child(format!("Page: {}", current_route.title())),
                                )
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .px(px(5.0))
                                        .py(px(1.0))
                                        .rounded(px(4.0))
                                        .bg(theme.card_bg)
                                        .border_1()
                                        .border_color(theme.card_border)
                                        .text_color(theme.text_primary)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(route_weight),
                                ),
                        )
                        // Telemetry Poller row + Toggle
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .child("Telemetry (500ms):"),
                                )
                                .child({
                                    let is_tel_hovered =
                                        snapshot.hovered_control == Some("telemetry");
                                    let tel_spring =
                                        hover_spring(if is_tel_hovered { 0.5 } else { 0.0 });
                                    let on_hover_tel = on_hover_control.clone();
                                    let tel_accent = if snapshot.freeze_telemetry {
                                        theme.accent_red
                                    } else {
                                        theme.accent_green
                                    };

                                    Chip::new(
                                        "dev_perf_freeze_telemetry_btn",
                                        if snapshot.freeze_telemetry {
                                            "PAUSED"
                                        } else {
                                            "ACTIVE"
                                        },
                                    )
                                    .destructive(snapshot.freeze_telemetry)
                                    .selected(!snapshot.freeze_telemetry)
                                    .spring(tel_spring, tel_accent)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_tel {
                                            h("telemetry", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_freeze_telemetry(window, cx);
                                        },
                                    )
                                }),
                        )
                        // 60 FPS Chart Animation Loop row + Toggle
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(theme.text_muted)
                                        .child("Chart Glide (16ms):"),
                                )
                                .child({
                                    let is_chart_hovered =
                                        snapshot.hovered_control == Some("chart_anim");
                                    let chart_spring =
                                        hover_spring(if is_chart_hovered { 0.5 } else { 0.0 });
                                    let on_hover_chart = on_hover_control.clone();
                                    let chart_accent = if snapshot.disable_chart_animation {
                                        theme.accent_red
                                    } else {
                                        theme.accent_green
                                    };

                                    Chip::new(
                                        "dev_perf_chart_anim_btn",
                                        if snapshot.disable_chart_animation {
                                            "OFF"
                                        } else {
                                            "ON"
                                        },
                                    )
                                    .destructive(snapshot.disable_chart_animation)
                                    .selected(!snapshot.disable_chart_animation)
                                    .spring(chart_spring, chart_accent)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_chart {
                                            h("chart_anim", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_chart_anim(window, cx);
                                        },
                                    )
                                }),
                        )
                        // Continuous Drive Mode row + Toggle (from longbridge/gpui-component/fps)
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .text_xs()
                                .child(div().text_color(theme.text_muted).child("Continuous Mode:"))
                                .child({
                                    let is_cont_hovered =
                                        snapshot.hovered_control == Some("continuous");
                                    let cont_spring =
                                        hover_spring(if is_cont_hovered { 0.5 } else { 0.0 });
                                    let on_hover_cont = on_hover_control.clone();

                                    Chip::new(
                                        "dev_perf_continuous_btn",
                                        if snapshot.continuous_mode {
                                            "DRIVING"
                                        } else {
                                            "IDLE-AWARE"
                                        },
                                    )
                                    .selected(snapshot.continuous_mode)
                                    .spring(cont_spring, theme.accent_blue)
                                    .on_hover(move |hov, window, cx| {
                                        if let Some(ref h) = on_hover_cont {
                                            h("continuous", hov, window, cx);
                                        }
                                    })
                                    .on_mouse_down(
                                        move |window, cx| {
                                            on_continuous(window, cx);
                                        },
                                    )
                                }),
                        )
}
