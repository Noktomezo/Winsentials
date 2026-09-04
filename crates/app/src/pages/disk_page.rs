use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString, Styled, Window,
    div, px,
};
use rust_i18n::t;

use crate::entities::hardware::{DiskInfo, DiskKind};
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;

use crate::shared::ui::history_graph::{HistoryGraphPalette, render_stepped_history_graph};

#[derive(IntoElement)]
pub struct DiskPage {
    disk: DiskInfo,
}

impl DiskPage {
    #[must_use]
    pub const fn new(disk: DiskInfo) -> Self {
        Self { disk }
    }
}

fn render_value(value: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(value.into())
}

fn render_info_row(
    label: impl Into<SharedString>,
    value: impl IntoElement,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .min_h(px(20.0))
        .child(
            div()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(value)
}

impl RenderOnce for DiskPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let disk = self.disk;
        let disk_name = disk
            .custom_name
            .as_deref()
            .map_or_else(|| disk.letter.to_string(), ToString::to_string);
        let detail = if disk.custom_name.is_some() {
            format!("({} - {disk_name})", disk.letter)
        } else {
            format!("({}:)", disk.letter)
        };
        let yes_no = |value| {
            if value {
                t!("disk_detail.yes")
            } else {
                t!("disk_detail.no")
            }
        };

        let header = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .text_size(px(20.0))
                    .line_height(px(24.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child(format!("{} {}", t!("telemetry.disk"), disk.id))
                    .child(
                        div()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_muted)
                            .child(detail),
                    ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(theme.text_muted)
                    .child(t!("disk_detail.desc")),
            );

        let graph_footer = || {
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .text_size(px(11.5))
                .text_color(theme.text_muted)
                .child(t!("disk_detail.graph_time"))
                .child("0")
        };

        let active_graph = render_stepped_history_graph(
            &disk.active_history_15s,
            None,
            disk.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "disk-active-graph-live-point",
            (100.0, 100.0),
            "%",
        );
        let active_card = GroupCard::new(
            "icons/activity.svg",
            t!("disk_detail.active_time"),
            t!("disk_detail.active_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .w_full()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .child(t!("disk_detail.active_time"))
                        .child("100%"),
                )
                .child(active_graph)
                .child(graph_footer()),
        );

        let transfer_graph = render_stepped_history_graph(
            &disk.transfer_history_15s,
            None,
            disk.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "disk-transfer-graph-live-point",
            (disk.previous_transfer_scale, disk.transfer_scale),
            format!(" {}", t!("disk_detail.mb_s")),
        );
        let transfer_card = GroupCard::new(
            "icons/arrow-left-right.svg",
            t!("disk_detail.transfer_rate"),
            t!("disk_detail.transfer_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .w_full()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .child(t!("disk_detail.transfer_rate"))
                        .child(format!(
                            "{:.0} {}",
                            disk.transfer_scale,
                            t!("disk_detail.mb_s")
                        )),
                )
                .child(transfer_graph)
                .child(graph_footer()),
        );

        let left_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("disk_detail.active_time"),
                render_value(format!("{:.0}%", disk.active_percent), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.read_speed"),
                render_value(
                    format!("{:.1} {}", disk.read_mb_s, t!("disk_detail.mb_s")),
                    theme.accent_blue,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.device"),
                render_value(disk_name, theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.format"),
                render_value(disk.file_system, theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.page_file"),
                render_value(yes_no(disk.is_system), theme.text_primary),
                &theme,
            ));

        let right_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("disk_detail.response_time"),
                render_value(
                    format!("{:.1} {}", disk.average_response_ms, t!("disk_detail.ms")),
                    theme.accent_blue,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.write_speed"),
                render_value(
                    format!("{:.1} {}", disk.write_mb_s, t!("disk_detail.mb_s")),
                    theme.accent_blue,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.capacity"),
                render_value(
                    format!("{} {}", disk.total_gb, t!("telemetry.gb")),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.system_disk"),
                render_value(yes_no(disk.is_system), theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("disk_detail.type"),
                render_value(
                    if disk.is_removable {
                        t!("disk_detail.removable")
                    } else {
                        match disk.kind {
                            DiskKind::NvmeSsd => t!("disk_detail.nvme_ssd"),
                            DiskKind::Ssd => t!("disk_detail.ssd"),
                            DiskKind::Hdd => t!("disk_detail.hdd"),
                            DiskKind::Unknown => t!("disk_detail.unknown_type"),
                        }
                    },
                    theme.text_primary,
                ),
                &theme,
            ));

        let info_card = GroupCard::new(
            "icons/hard-drive.svg",
            t!("disk_detail.info_title"),
            t!("disk_detail.info_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(
            div()
                .flex()
                .gap(px(20.0))
                .w_full()
                .child(left_info)
                .child(right_info),
        );

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(header)
            .child(active_card)
            .child(transfer_card)
            .child(info_card)
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::hardware::transfer_scale;

    #[test]
    fn transfer_scale_has_a_readable_floor_and_grows_with_traffic() {
        assert_eq!(transfer_scale(&[0.0, 3.0]), 50.0);
        assert_eq!(transfer_scale(&[101.0]), 110.0);
    }
}
