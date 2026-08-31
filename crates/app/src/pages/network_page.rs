use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString, Styled, Window,
    div, px,
};
use rust_i18n::t;

use crate::entities::hardware::{NetworkInfo, NetworkKind};
use crate::shared::theme::Theme;
use crate::shared::ui::{GroupCard, Icon};

use super::cpu_page::{HistoryGraphPalette, render_stepped_history_graph};
use super::page_header::PageHeader;

#[derive(IntoElement)]
pub struct NetworkPage {
    network: NetworkInfo,
}

impl NetworkPage {
    #[must_use]
    pub const fn new(network: NetworkInfo) -> Self {
        Self { network }
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
            .w_full()
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

    fn render_legend(
        icon: &'static str,
        label: impl Into<SharedString>,
        value: SharedString,
        color: Rgba,
        theme: &Theme,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .child(Icon::new(icon).size(px(13.0)).color(color))
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(theme.text_muted)
                    .child(label.into()),
            )
            .child(Self::render_value(value, color))
    }
}

fn format_link_speed(mbps: u64) -> String {
    if mbps >= 1_000 {
        let whole = mbps / 1_000;
        let tenths = mbps % 1_000 / 100;
        format!("{whole}.{tenths} {}", t!("network_detail.gbps"))
    } else {
        format!("{mbps} {}", t!("network_detail.mbps"))
    }
}

impl RenderOnce for NetworkPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let network = self.network;
        let connection_type = match network.kind {
            NetworkKind::Ethernet => t!("network_detail.ethernet"),
            NetworkKind::Wifi => t!("network_detail.wifi"),
        };
        let rx_speed = network.rx_speed.clone();
        let tx_speed = network.tx_speed.clone();

        let graph = render_stepped_history_graph(
            &network.rx_history_15s,
            Some((&network.tx_history_15s, theme.accent_cyan)),
            network.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "network-throughput-graph-live-point",
            (network.previous_throughput_scale, network.throughput_scale),
            format!(" {}", t!("network_detail.mbps")),
        );

        let graph_card = GroupCard::new(
            "icons/activity.svg",
            t!("network_detail.throughput"),
            t!("network_detail.throughput_desc"),
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
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(18.0))
                                .child(Self::render_legend(
                                    "icons/arrow-down.svg",
                                    t!("network_detail.receive"),
                                    rx_speed.clone(),
                                    theme.accent_blue,
                                    &theme,
                                ))
                                .child(Self::render_legend(
                                    "icons/arrow-up.svg",
                                    t!("network_detail.send"),
                                    tx_speed.clone(),
                                    theme.accent_cyan,
                                    &theme,
                                )),
                        )
                        .child(Self::render_value(
                            format!(
                                "{:.1} {}",
                                network.throughput_scale,
                                t!("network_detail.mbps")
                            ),
                            theme.text_muted,
                        )),
                )
                .child(graph)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .text_size(px(11.5))
                        .text_color(theme.text_muted)
                        .child(t!("network_detail.graph_time"))
                        .child("0"),
                ),
        );

        let left_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(Self::render_info_row(
                t!("network_detail.adapter"),
                Self::render_value(network.adapter_name.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("network_detail.interface"),
                Self::render_value(network.interface_name.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("network_detail.connection_type"),
                Self::render_value(connection_type, theme.text_primary),
                &theme,
            ));

        let right_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(Self::render_info_row(
                t!("network_detail.link_speed"),
                Self::render_value(
                    format_link_speed(network.link_speed_mbps),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("network_detail.receive"),
                Self::render_value(rx_speed, theme.accent_blue),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("network_detail.send"),
                Self::render_value(tx_speed, theme.accent_cyan),
                &theme,
            ));

        let info_card = GroupCard::new(
            "icons/network.svg",
            t!("network_detail.info_title"),
            t!("network_detail.info_desc"),
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
            .child(PageHeader::new(
                network.interface_name,
                network.adapter_name,
            ))
            .child(graph_card)
            .child(info_card)
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::hardware::throughput_scale;

    #[test]
    fn throughput_scale_keeps_idle_visible_and_adds_headroom() {
        assert_eq!(throughput_scale(&[0.0], &[0.0]), 1.0);
        assert_eq!(throughput_scale(&[7.0], &[2.0]), 16.0);
    }
}
