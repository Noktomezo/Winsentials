use std::sync::Arc;

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString, Styled, Window,
    div, px,
};
use rust_i18n::t;

use crate::entities::hardware::CpuDetailData;
use crate::features::navigation::AppRoute;
use crate::shared::theme::Theme;
use crate::shared::ui::GroupCard;

pub type NavigateHandler = Arc<dyn Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static>;

#[derive(IntoElement)]
pub struct CpuPage {
    cpu_detail: CpuDetailData,
    on_navigate: Option<NavigateHandler>,
}

impl CpuPage {
    #[must_use]
    pub fn new(cpu_detail: CpuDetailData) -> Self {
        Self {
            cpu_detail,
            on_navigate: None,
        }
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
        .py(px(3.0))
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

fn render_text_val(text: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .text_ellipsis()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(text.into())
}

pub use crate::shared::ui::history_graph::{
    HistoryGraphPalette, graph_percent_color, history_graph_color, render_stepped_history_graph,
    render_stepped_history_graph_sized, smooth_percent_transition, stepped_corner_radius,
    stepped_history_index,
};

const CORE_GRID_COLUMNS: usize = 4;

fn core_grid_placeholder_count(item_count: usize) -> usize {
    (CORE_GRID_COLUMNS - item_count % CORE_GRID_COLUMNS) % CORE_GRID_COLUMNS
}

fn render_core_card(core_idx: usize, utilization_pct: f32, theme: &Theme) -> impl IntoElement {
    let color = if utilization_pct > 85.0 {
        theme.accent_red
    } else if utilization_pct >= 60.0 {
        theme.accent_yellow
    } else {
        theme.accent_green
    };

    let fill_ratio = (utilization_pct / 100.0).clamp(0.0, 1.0);

    div()
        .flex()
        .flex_col()
        .justify_between()
        .gap(px(6.0))
        .p(px(10.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(theme.card_border)
        .bg(theme.input_bg)
        .min_w(px(140.0))
        .flex_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_primary)
                        .child(format!("{} {}", t!("cpu_detail.core_label"), core_idx + 1)),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(color)
                        .child(format!("{utilization_pct:.0}%")),
                ),
        )
        .child(
            div()
                .h(px(6.0))
                .w_full()
                .rounded(px(3.0))
                .bg(theme.card_bg)
                .border_1()
                .border_color(theme.card_border)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(gpui::DefiniteLength::Fraction(fill_ratio))
                        .rounded(px(3.0))
                        .bg(color),
                ),
        )
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::{
        core_grid_placeholder_count, graph_percent_color, smooth_percent_transition,
        stepped_corner_radius, stepped_history_index,
    };
    use crate::shared::theme::Theme;

    #[test]
    fn graph_colors_blend_between_dashboard_zones() {
        let theme = Theme::dark();

        assert_eq!(graph_percent_color(0.0, &theme), theme.accent_green);
        assert_eq!(graph_percent_color(55.0, &theme), theme.accent_green);
        assert_ne!(graph_percent_color(60.0, &theme), theme.accent_green);
        assert_ne!(graph_percent_color(60.0, &theme), theme.accent_yellow);
        assert_eq!(graph_percent_color(65.0, &theme), theme.accent_yellow);
        assert_eq!(graph_percent_color(80.0, &theme), theme.accent_yellow);
        assert_eq!(graph_percent_color(90.0, &theme), theme.accent_red);
    }

    #[test]
    fn core_transition_is_smooth_and_clamped() {
        assert_eq!(smooth_percent_transition(20.0, 80.0, -1.0), 20.0);
        assert_eq!(smooth_percent_transition(20.0, 80.0, 0.5), 50.0);
        assert_eq!(smooth_percent_transition(20.0, 80.0, 2.0), 80.0);
    }

    #[test]
    fn core_grid_keeps_four_equal_slots_per_row() {
        assert_eq!(core_grid_placeholder_count(4), 0);
        assert_eq!(core_grid_placeholder_count(3), 1);
        assert_eq!(core_grid_placeholder_count(1), 3);
    }

    #[test]
    fn rounded_step_radius_stays_inside_short_transitions() {
        assert_eq!(
            stepped_corner_radius(px(12.0), px(20.0), px(20.0)).height,
            px(0.0)
        );
        assert_eq!(
            stepped_corner_radius(px(12.0), px(20.0), px(22.0)).height,
            px(1.0)
        );
    }

    #[test]
    fn maps_cursor_to_stepped_history_segment() {
        assert_eq!(stepped_history_index(0.0, 30), 0);
        assert_eq!(stepped_history_index(1.1, 30), 1);
        assert_eq!(stepped_history_index(99.0, 30), 29);
    }
}

impl RenderOnce for CpuPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let cpu = self.cpu_detail;

        // Header stack: Title with full CPU Model in muted text alongside
        let header_stack = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(20.0))
                            .line_height(px(24.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(t!("cpu_detail.title")),
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .line_height(px(20.0))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_muted)
                            .child(format!("({})", cpu.model)),
                    ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(t!("cpu_detail.desc")),
            );

        // 2. Card 1: 60-Second Utilization Graph Card
        let graph_top_row = div()
            .flex()
            .items_baseline()
            .justify_between()
            .w_full()
            .child(
                div()
                    .text_size(px(12.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_muted)
                    .child(t!("cpu_detail.utilization")),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_muted)
                    .child("100%"),
            );

        let graph_bottom_row = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(t!("cpu_detail.graph_time")),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child("0"),
            );

        let graph_canvas_box = render_stepped_history_graph(
            &cpu.history_15s,
            None,
            cpu.sample_instant,
            &theme,
            HistoryGraphPalette::Semantic,
            "cpu-graph-live-point",
            (100.0, 100.0),
            "%",
        );

        let graph_card = GroupCard::new(
            "icons/activity.svg",
            t!("cpu_detail.utilization"),
            t!("cpu_detail.desc"),
        )
        .icon_color(theme.accent_blue)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .w_full()
                .child(graph_top_row)
                .child(graph_canvas_box)
                .child(graph_bottom_row),
        );

        // 3. Card 2: Processor Information Card (2 Columns)
        let left_info_col = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("cpu_detail.model"),
                render_text_val(cpu.model.clone(), theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.processes"),
                render_text_val(cpu.processes.to_string(), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.threads"),
                render_text_val(cpu.threads.to_string(), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.handles"),
                render_text_val(cpu.handles.to_string(), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.uptime"),
                render_text_val(cpu.uptime.clone(), theme.accent_blue),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.base_frequency"),
                render_text_val(
                    format!("{:.2} {}", cpu.base_clock_ghz, t!("cpu_detail.ghz")),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.current_frequency"),
                render_text_val(
                    format!("{:.2} {}", cpu.current_clock_ghz, t!("cpu_detail.ghz")),
                    theme.accent_blue,
                ),
                &theme,
            ));

        let virt_label = if cpu.virtualization {
            t!("cpu_detail.virtualization_enabled")
        } else {
            t!("cpu_detail.virtualization_disabled")
        };

        let right_info_col = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(render_info_row(
                t!("cpu_detail.sockets"),
                render_text_val(cpu.sockets.to_string(), theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.cores"),
                render_text_val(cpu.cores.to_string(), theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.logical_processors"),
                render_text_val(cpu.logical_processors.to_string(), theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.virtualization"),
                render_text_val(virt_label, theme.text_primary),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.cache_l1"),
                render_text_val(
                    format!("{} {}", cpu.l1_cache_kb, t!("cpu_detail.kb")),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.cache_l2"),
                render_text_val(
                    format!("{} {}", cpu.l2_cache_mb, t!("cpu_detail.mb")),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(render_info_row(
                t!("cpu_detail.cache_l3"),
                render_text_val(
                    format!("{} {}", cpu.l3_cache_mb, t!("cpu_detail.mb")),
                    theme.text_primary,
                ),
                &theme,
            ));

        let info_grid_row = div()
            .flex()
            .flex_row()
            .gap(px(20.0))
            .w_full()
            .child(left_info_col)
            .child(right_info_col);

        let info_card = GroupCard::new(
            "icons/monitor.svg",
            t!("cpu_detail.info_title"),
            t!("cpu_detail.info_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(info_grid_row);

        // 4. Card 3: Per-Core Utilization Grid Card
        let mut cores_grid = div().flex().flex_col().gap(px(10.0)).w_full();
        let core_transition_progress = cpu.sample_instant.elapsed().as_secs_f32() / 0.5;

        for (row_index, cores) in cpu.core_utilization.chunks(CORE_GRID_COLUMNS).enumerate() {
            let mut row = div().flex().gap(px(10.0)).w_full();

            for (column_index, &core_val) in cores.iter().enumerate() {
                let idx = row_index * CORE_GRID_COLUMNS + column_index;
                let previous = cpu
                    .previous_core_utilization
                    .get(idx)
                    .copied()
                    .unwrap_or(core_val);
                let displayed =
                    smooth_percent_transition(previous, core_val, core_transition_progress);
                row = row.child(render_core_card(idx, displayed, &theme));
            }

            for _ in 0..core_grid_placeholder_count(cores.len()) {
                row = row.child(div().min_w(px(140.0)).flex_1());
            }

            cores_grid = cores_grid.child(row);
        }

        let cores_card = GroupCard::new(
            "icons/cpu.svg",
            t!("cpu_detail.cores_title"),
            t!("cpu_detail.cores_desc"),
        )
        .icon_color(theme.accent_blue)
        .child(cores_grid);

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(header_stack)
            .child(graph_card)
            .child(info_card)
            .child(cores_card)
    }
}
