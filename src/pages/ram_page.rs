use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, DefiniteLength, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString,
    Styled, Window, div, px,
};
use rust_i18n::t;

use crate::entities::hardware::RamInfo;
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;

use super::cpu_page::{HistoryGraphPalette, render_stepped_history_graph};

#[derive(IntoElement)]
pub struct RamPage {
    ram: RamInfo,
}

impl RamPage {
    #[must_use]
    pub fn new(ram: RamInfo) -> Self {
        Self { ram }
    }
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
        .min_h(px(24.0))
        .w_full()
        .child(
            div()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(value)
}

fn render_value(text: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .whitespace_nowrap()
        .child(text.into())
}

fn render_legend_item(
    label: impl Into<SharedString>,
    value: String,
    color: Rgba,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(div().size(px(8.0)).rounded(px(4.0)).bg(color))
        .child(
            div()
                .text_size(px(11.5))
                .text_color(theme.text_muted)
                .child(label.into()),
        )
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(value),
        )
}

fn gb(value: f32) -> String {
    format!("{value:.1} {}", t!("ram_detail.gb"))
}

fn mb(value: f32) -> String {
    format!("{value:.0} {}", t!("ram_detail.mb"))
}

impl RenderOnce for RamPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let ram = self.ram;
        let total = ram.total_gb.max(0.001);
        let cached_gb = (ram.cached_mb / 1024.0).clamp(0.0, ram.available_gb);
        let free_gb = (ram.available_gb - cached_gb).max(0.0);
        let has_cached = cached_gb > 0.001;
        let has_free = free_gb > 0.001;

        let header = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(
                div()
                    .text_size(px(20.0))
                    .line_height(px(24.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child(t!("ram_detail.title")),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(theme.text_muted)
                    .child(t!("ram_detail.desc")),
            );

        let graph = render_stepped_history_graph(
            &ram.history_15s,
            None,
            ram.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "ram-graph-live-point",
            (100.0, 100.0),
            "%",
        );
        let graph_card = GroupCard::new(
            "icons/activity.svg",
            t!("ram_detail.usage_title"),
            t!("ram_detail.usage_desc"),
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
                                .text_size(px(12.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_muted)
                                .child(t!("ram_detail.used")),
                        )
                        .child(render_value(
                            format!("{} / {}", gb(ram.used_gb), gb(ram.total_gb)),
                            theme.accent_blue,
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
                        .child(t!("ram_detail.graph_time"))
                        .child("0"),
                ),
        );

        let composition_card = GroupCard::new(
            "icons/chart-bar-stacked.svg",
            t!("ram_detail.composition_title"),
            t!("ram_detail.composition_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .w_full()
                .child(
                    div()
                        .flex()
                        .h(px(16.0))
                        .w_full()
                        .rounded(px(6.0))
                        .overflow_hidden()
                        .bg(theme.card_bg)
                        .child(
                            div()
                                .h_full()
                                .w(DefiniteLength::Fraction(ram.used_gb / total))
                                .rounded_tl(px(6.0))
                                .rounded_bl(px(6.0))
                                .when(!has_cached && !has_free, |segment| {
                                    segment.rounded_tr(px(6.0)).rounded_br(px(6.0))
                                })
                                .bg(theme.accent_blue),
                        )
                        .child(
                            div()
                                .h_full()
                                .w(DefiniteLength::Fraction(cached_gb / total))
                                .when(!has_free, |segment| {
                                    segment.rounded_tr(px(6.0)).rounded_br(px(6.0))
                                })
                                .bg(theme.accent_yellow),
                        )
                        .child(
                            div()
                                .h_full()
                                .w(DefiniteLength::Fraction(free_gb / total))
                                .rounded_tr(px(6.0))
                                .rounded_br(px(6.0))
                                .bg(theme.accent_green),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_x(px(18.0))
                        .gap_y(px(6.0))
                        .child(render_legend_item(
                            t!("ram_detail.used"),
                            gb(ram.used_gb),
                            theme.accent_blue,
                            &theme,
                        ))
                        .child(render_legend_item(
                            t!("ram_detail.cached"),
                            gb(cached_gb),
                            theme.accent_yellow,
                            &theme,
                        ))
                        .child(render_legend_item(
                            t!("ram_detail.free"),
                            gb(free_gb),
                            theme.accent_green,
                            &theme,
                        )),
                ),
        );

        let left_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("ram_detail.used"),
                render_value(gb(ram.used_gb), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.committed"),
                render_value(
                    format!("{} / {}", gb(ram.committed_gb), gb(ram.commit_limit_gb)),
                    theme.accent_blue,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.paged_pool"),
                render_value(mb(ram.paged_pool_mb), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.speed"),
                render_value(
                    format!("{} {}", ram.speed_mhz, t!("ram_detail.mhz")),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.form_factor"),
                render_value(ram.form_factor, theme.text_primary),
                &theme,
            ));

        let right_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("ram_detail.available"),
                render_value(gb(ram.available_gb), theme.accent_green),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.cached"),
                render_value(mb(ram.cached_mb), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.non_paged_pool"),
                render_value(mb(ram.non_paged_pool_mb), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.slots"),
                render_value(ram.slots, theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("ram_detail.hardware_reserved"),
                render_value(mb(ram.hardware_reserved_mb), theme.accent_yellow),
                &theme,
            ));

        let info_card = GroupCard::new(
            "icons/memory-stick.svg",
            t!("ram_detail.info_title"),
            t!("ram_detail.info_desc"),
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
            .child(graph_card)
            .child(composition_card)
            .child(info_card)
    }
}
