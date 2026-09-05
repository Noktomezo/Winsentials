use gpui::{
    FontWeight, IntoElement, ParentElement, Rgba, SharedString, Styled, div, px,
};
use rust_i18n::t;

use crate::entities::hardware::GpuInfo;
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;

#[must_use]
pub(crate) fn semantic_percent_color(pct: f32, theme: &Theme) -> Rgba {
    if pct > 85.0 {
        theme.accent_red
    } else if pct >= 60.0 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

#[must_use]
pub(crate) fn semantic_temp_color(temp_c: u32, theme: &Theme) -> Rgba {
    if temp_c >= 80 {
        theme.accent_red
    } else if temp_c >= 65 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

pub(crate) fn render_value(value: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(value.into())
}

pub(crate) fn render_info_row(
    label: impl Into<SharedString>,
    value: impl IntoElement,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .min_h(px(24.0))
        .w_full()
        .child(
            div()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(div().flex_none().child(value))
}

pub(crate) fn render_gpu_info_card(gpu: &GpuInfo, theme: &Theme) -> impl IntoElement {
    let memory_used_gb = gpu.memory_used_mb / 1024.0;
    let memory_total_gb = gpu.memory_total_mb / 1024.0;
    let dedicated_used_gb = gpu.dedicated_used_mb / 1024.0;
    let dedicated_total_gb = gpu.dedicated_total_mb / 1024.0;
    let shared_used_gb = gpu.shared_used_mb / 1024.0;
    let shared_total_gb = gpu.shared_total_mb / 1024.0;

    let left_info = div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .flex_1()
        .min_w(px(0.0))
        .child(render_info_row(
            t!("gpu_detail.utilization"),
            render_value(
                format!("{}%", gpu.usage_percent),
                semantic_percent_color(
                    f32::from(u16::try_from(gpu.usage_percent).unwrap_or(0)),
                    theme,
                ),
            ),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.gpu_memory"),
            render_value(
                format!(
                    "{memory_used_gb:.1} / {memory_total_gb:.1} {}",
                    t!("telemetry.gb")
                ),
                theme.text_primary,
            ),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.dedicated_gpu_memory"),
            render_value(
                format!(
                    "{dedicated_used_gb:.1} / {dedicated_total_gb:.1} {}",
                    t!("telemetry.gb")
                ),
                theme.text_primary,
            ),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.shared_gpu_memory"),
            render_value(
                format!(
                    "{shared_used_gb:.1} / {shared_total_gb:.1} {}",
                    t!("telemetry.gb")
                ),
                theme.text_primary,
            ),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.temperature"),
            render_value(
                format!("{} °C", gpu.temperature_c),
                semantic_temp_color(gpu.temperature_c, theme),
            ),
            theme,
        ));

    let right_info = div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .flex_1()
        .min_w(px(0.0))
        .child(render_info_row(
            t!("gpu_detail.driver_version"),
            render_value(gpu.driver_version.clone(), theme.text_primary),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.driver_date"),
            render_value(gpu.driver_date.clone(), theme.text_primary),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.directx_version"),
            render_value(gpu.directx_version.clone(), theme.text_primary),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.physical_location"),
            render_value(gpu.pci_location.clone(), theme.text_primary),
            theme,
        ))
        .child(render_info_row(
            t!("gpu_detail.hardware_reserved"),
            render_value(
                format!("{} {}", gpu.hardware_reserved_mb, t!("gpu_detail.mb")),
                theme.text_primary,
            ),
            theme,
        ));

    GroupCard::new(
        "icons/circuit-board.svg",
        t!("gpu_detail.info_title"),
        t!("gpu_detail.info_desc"),
    )
    .icon_color(theme.accent_blue)
    .child(
        div()
            .flex()
            .gap(px(32.0))
            .w_full()
            .child(left_info)
            .child(right_info),
    )
}