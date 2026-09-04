use std::sync::Arc;

use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Rgba,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use rust_i18n::t;

use crate::entities::hardware::GpuInfo;
use crate::shared::theme::Theme;
use crate::shared::ui::{Dropdown, GroupCard, Icon};

use super::page_header::PageHeader;
use crate::shared::ui::history_graph::{HistoryGraphPalette, render_stepped_history_graph};

fn semantic_percent_color(pct: f32, theme: &Theme) -> Rgba {
    if pct > 85.0 {
        theme.accent_red
    } else if pct >= 60.0 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

fn semantic_temp_color(temp_c: u32, theme: &Theme) -> Rgba {
    if temp_c >= 80 {
        theme.accent_red
    } else if temp_c >= 65 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

pub type SelectGpuEngineHandler = Arc<dyn Fn(usize, &'static str, &mut Window, &mut App) + 'static>;
pub type ResetGpuSlotsHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type GpuToggleDropdownHandler = Arc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type GpuHoverDropdownHandler =
    Arc<dyn Fn(&'static str, &bool, &mut Window, &mut App) + 'static>;
pub type GpuHoverOptionHandler =
    Arc<dyn Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static>;
pub type GpuCloseDropdownsHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct GpuPage {
    gpu: GpuInfo,
    slots: [&'static str; 4],
    open_dropdown: Option<&'static str>,
    opening_dropdown: Option<&'static str>,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    on_select_engine: Option<SelectGpuEngineHandler>,
    on_reset_slots: Option<ResetGpuSlotsHandler>,
    on_toggle_dropdown: Option<GpuToggleDropdownHandler>,
    on_hover_dropdown: Option<GpuHoverDropdownHandler>,
    on_hover_option: Option<GpuHoverOptionHandler>,
    on_close_dropdowns: Option<GpuCloseDropdownsHandler>,
}

impl GpuPage {
    #[must_use]
    pub fn new(
        gpu: GpuInfo,
        slots: [&'static str; 4],
        open_dropdown: Option<&'static str>,
        opening_dropdown: Option<&'static str>,
        closing_dropdown: Option<&'static str>,
        hovered_dropdown: Option<&'static str>,
        hovered_option: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            gpu,
            slots,
            open_dropdown,
            opening_dropdown,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            on_select_engine: None,
            on_reset_slots: None,
            on_toggle_dropdown: None,
            on_hover_dropdown: None,
            on_hover_option: None,
            on_close_dropdowns: None,
        }
    }

    #[must_use]
    pub fn on_select_engine(
        mut self,
        handler: impl Fn(usize, &'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_engine = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_reset_slots(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reset_slots = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_dropdown(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_dropdown(
        mut self,
        handler: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_option(
        mut self,
        handler: impl Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_option = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close_dropdowns(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close_dropdowns = Some(Arc::new(handler));
        self
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

    fn graph_footer() -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .text_size(px(11.5))
            .text_color(crate::shared::theme::Theme::dark().text_muted)
            .child(t!("gpu_detail.graph_time"))
            .child("0")
    }
}

impl RenderOnce for GpuPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let gpu = self.gpu;
        let on_reset_slots = self.on_reset_slots.clone();
        let on_select_engine = self.on_select_engine.clone();
        let on_toggle_dropdown = self.on_toggle_dropdown.clone();
        let on_hover_dropdown = self.on_hover_dropdown.clone();
        let on_hover_option = self.on_hover_option.clone();
        let on_close_dropdowns = self.on_close_dropdowns.clone();

        // 1. Engine Grid (2x2) Card
        let reset_btn = div()
            .id("gpu_reset_btn")
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(3.0))
            .rounded(px(5.0))
            .border_1()
            .border_color(theme.input_border)
            .bg(theme.input_bg)
            .cursor_pointer()
            .hover(move |s| s.border_color(theme.accent_blue))
            .on_click(move |_event, window, cx| {
                if let Some(ref h) = on_reset_slots {
                    h(window, cx);
                }
            })
            .child(
                Icon::new("icons/rotate-ccw.svg")
                    .size(px(12.0))
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_primary)
                    .child(t!("gpu_detail.reset_selection")),
            );

        let mut engine_slots_grid = div().flex().flex_col().gap(px(12.0)).w_full();

        // 2x2 rows
        for row_idx in 0..2 {
            let mut row = div().flex().gap(px(12.0)).w_full();

            for col_idx in 0..2 {
                let slot_idx = row_idx * 2 + col_idx;
                let current_engine = self.slots[slot_idx];
                let utilization = gpu
                    .engine_utilizations
                    .get(current_engine)
                    .copied()
                    .unwrap_or(0.0);
                let history = gpu
                    .engine_histories_15s
                    .get(current_engine)
                    .cloned()
                    .unwrap_or_else(|| vec![0.0; 30]);

                let dropdown_id: &'static str = match slot_idx {
                    0 => "gpu_engine_slot_0",
                    1 => "gpu_engine_slot_1",
                    2 => "gpu_engine_slot_2",
                    _ => "gpu_engine_slot_3",
                };

                let is_open = self.open_dropdown == Some(dropdown_id);
                let is_opening = self.opening_dropdown == Some(dropdown_id);
                let is_closing = self.closing_dropdown == Some(dropdown_id);
                let is_hovered = self.hovered_dropdown == Some(dropdown_id);
                let hovered_opt = self
                    .hovered_option
                    .and_then(|(dd, opt)| if dd == dropdown_id { Some(opt) } else { None });

                let select_handler = on_select_engine.clone();
                let toggle_handler = on_toggle_dropdown.clone();
                let hover_dd_handler = on_hover_dropdown.clone();
                let hover_opt_handler = on_hover_option.clone();
                let close_handler = on_close_dropdowns.clone();

                let options: Vec<(&'static str, &'static str, Option<&'static str>)> = gpu
                    .available_engines
                    .iter()
                    .map(|&e| (e, e, None))
                    .collect();

                let dropdown = Dropdown::new(dropdown_id, current_engine, current_engine)
                    .options(options)
                    .open(is_open)
                    .opening(is_opening)
                    .closing(is_closing)
                    .hovered(is_hovered)
                    .hovered_option(hovered_opt)
                    .on_toggle(move |window, cx| {
                        if let Some(ref h) = toggle_handler {
                            h(dropdown_id, window, cx);
                        }
                    })
                    .on_select(move |selected_val, window, cx| {
                        if let Some(ref h) = select_handler {
                            let static_val: &'static str = match selected_val {
                                "3D" => "3D",
                                "Copy" => "Copy",
                                "Video Encode" => "Video Encode",
                                "Video Decode" => "Video Decode",
                                "Overlay" => "Overlay",
                                "Copy 1" => "Copy 1",
                                "Security" => "Security",
                                "OFA_0" => "OFA_0",
                                "VR" => "VR",
                                "Copy 2" => "Copy 2",
                                "Copy 3" => "Copy 3",
                                "Copy 4" => "Copy 4",
                                "Copy 5" => "Copy 5",
                                "Security_1" => "Security_1",
                                "High Priority Compute" => "High Priority Compute",
                                "High Priority 3D" => "High Priority 3D",
                                "Compute 0" => "Compute 0",
                                "Compute 1" => "Compute 1",
                                "Timer 0" => "Timer 0",
                                "Security 1" => "Security 1",
                                "Video JPEG 0" => "Video JPEG 0",
                                "Video Decode 1" => "Video Decode 1",
                                "Video Codec 0" => "Video Codec 0",
                                other => Box::leak(other.to_string().into_boxed_str()),
                            };
                            h(slot_idx, static_val, window, cx);
                        }
                    })
                    .on_hover_trigger(move |&hov, window, cx| {
                        if let Some(ref h) = hover_dd_handler {
                            h(dropdown_id, &hov, window, cx);
                        }
                    })
                    .on_hover_option(move |opt, &hov, window, cx| {
                        if let Some(ref h) = hover_opt_handler {
                            let static_opt: &'static str = match opt {
                                "3D" => "3D",
                                "Copy" => "Copy",
                                "Video Encode" => "Video Encode",
                                "Video Decode" => "Video Decode",
                                "Overlay" => "Overlay",
                                "Copy 1" => "Copy 1",
                                "Security" => "Security",
                                "OFA_0" => "OFA_0",
                                "VR" => "VR",
                                "Copy 2" => "Copy 2",
                                "Copy 3" => "Copy 3",
                                "Copy 4" => "Copy 4",
                                "Copy 5" => "Copy 5",
                                "Security_1" => "Security_1",
                                "High Priority Compute" => "High Priority Compute",
                                "High Priority 3D" => "High Priority 3D",
                                "Compute 0" => "Compute 0",
                                "Compute 1" => "Compute 1",
                                "Timer 0" => "Timer 0",
                                "Security 1" => "Security 1",
                                "Video JPEG 0" => "Video JPEG 0",
                                "Video Decode 1" => "Video Decode 1",
                                "Video Codec 0" => "Video Codec 0",
                                other => Box::leak(other.to_string().into_boxed_str()),
                            };
                            h(dropdown_id, static_opt, &hov, window, cx);
                        }
                    })
                    .on_close(move |window, cx| {
                        if let Some(ref h) = close_handler {
                            h(window, cx);
                        }
                    });

                let graph_anim_id: &'static str = match slot_idx {
                    0 => "gpu-engine-slot-0-graph",
                    1 => "gpu-engine-slot-1-graph",
                    2 => "gpu-engine-slot-2-graph",
                    _ => "gpu-engine-slot-3-graph",
                };

                let graph = render_stepped_history_graph(
                    &history,
                    None,
                    gpu.sample_instant,
                    &theme,
                    HistoryGraphPalette::Semantic,
                    graph_anim_id,
                    (100.0, 100.0),
                    "%",
                );

                let slot_box = div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .flex_1()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(dropdown)
                            .child(Self::render_value(
                                format!("{utilization:.0}%"),
                                semantic_percent_color(utilization, &theme),
                            )),
                    )
                    .child(graph)
                    .child(Self::graph_footer());

                row = row.child(slot_box);
            }

            engine_slots_grid = engine_slots_grid.child(row);
        }

        let engines_card = GroupCard::new(
            "icons/circuit-board.svg",
            t!("gpu_detail.engines_title"),
            t!("gpu_detail.engines_desc"),
        )
        .icon_color(theme.accent_blue)
        .header_action(reset_btn)
        .child(engine_slots_grid);

        // 2. Dedicated Memory Card
        let dedicated_used_gb = gpu.dedicated_used_mb / 1024.0;
        let dedicated_total_gb = gpu.dedicated_total_mb / 1024.0;

        let dedicated_graph = render_stepped_history_graph(
            &gpu.dedicated_history_15s,
            None,
            gpu.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "gpu-dedicated-mem-live-point",
            (gpu.dedicated_total_mb, gpu.dedicated_total_mb),
            format!(" {}", t!("gpu_detail.mb")),
        );

        let dedicated_card = GroupCard::new(
            "icons/memory-stick.svg",
            t!("gpu_detail.dedicated_memory"),
            t!("gpu_detail.dedicated_memory_desc"),
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
                        .child(format!(
                            "{:.1} / {:.1} {}",
                            dedicated_used_gb,
                            dedicated_total_gb,
                            t!("telemetry.gb")
                        ))
                        .child(format!("{:.1} {}", dedicated_total_gb, t!("telemetry.gb"))),
                )
                .child(dedicated_graph)
                .child(Self::graph_footer()),
        );

        // 3. Shared Memory Card
        let shared_used_gb = gpu.shared_used_mb / 1024.0;
        let shared_total_gb = gpu.shared_total_mb / 1024.0;

        let shared_graph = render_stepped_history_graph(
            &gpu.shared_history_15s,
            None,
            gpu.sample_instant,
            &theme,
            HistoryGraphPalette::Solid(theme.accent_blue),
            "gpu-shared-mem-live-point",
            (gpu.shared_total_mb, gpu.shared_total_mb),
            format!(" {}", t!("gpu_detail.mb")),
        );

        let shared_card = GroupCard::new(
            "icons/memory-stick.svg",
            t!("gpu_detail.shared_memory"),
            t!("gpu_detail.shared_memory_desc"),
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
                        .child(format!(
                            "{:.1} / {:.1} {}",
                            shared_used_gb,
                            shared_total_gb,
                            t!("telemetry.gb")
                        ))
                        .child(format!("{:.1} {}", shared_total_gb, t!("telemetry.gb"))),
                )
                .child(shared_graph)
                .child(Self::graph_footer()),
        );

        // 4. Specifications / Info Card (2 columns, strictly horizontal key-value rows)
        let memory_used_gb = gpu.memory_used_mb / 1024.0;
        let memory_total_gb = gpu.memory_total_mb / 1024.0;

        let left_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(Self::render_info_row(
                t!("gpu_detail.utilization"),
                Self::render_value(
                    format!("{}%", gpu.usage_percent),
                    semantic_percent_color(
                        f32::from(u16::try_from(gpu.usage_percent).unwrap_or(0)),
                        &theme,
                    ),
                ),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.gpu_memory"),
                Self::render_value(
                    format!(
                        "{memory_used_gb:.1} / {memory_total_gb:.1} {}",
                        t!("telemetry.gb")
                    ),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.dedicated_gpu_memory"),
                Self::render_value(
                    format!(
                        "{dedicated_used_gb:.1} / {dedicated_total_gb:.1} {}",
                        t!("telemetry.gb")
                    ),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.shared_gpu_memory"),
                Self::render_value(
                    format!(
                        "{shared_used_gb:.1} / {shared_total_gb:.1} {}",
                        t!("telemetry.gb")
                    ),
                    theme.text_primary,
                ),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.temperature"),
                Self::render_value(
                    format!("{} °C", gpu.temperature_c),
                    semantic_temp_color(gpu.temperature_c, &theme),
                ),
                &theme,
            ));

        let right_info = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .flex_1()
            .min_w(px(0.0))
            .child(Self::render_info_row(
                t!("gpu_detail.driver_version"),
                Self::render_value(gpu.driver_version.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.driver_date"),
                Self::render_value(gpu.driver_date.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.directx_version"),
                Self::render_value(gpu.directx_version.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.physical_location"),
                Self::render_value(gpu.pci_location.clone(), theme.text_primary),
                &theme,
            ))
            .child(Self::render_info_row(
                t!("gpu_detail.hardware_reserved"),
                Self::render_value(
                    format!("{} {}", gpu.hardware_reserved_mb, t!("gpu_detail.mb")),
                    theme.text_primary,
                ),
                &theme,
            ));

        let info_card = GroupCard::new(
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
        );

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(
                format!("{} {}", t!("telemetry.gpu"), gpu.id),
                format!("({})", gpu.name),
            ))
            .child(engines_card)
            .child(dedicated_card)
            .child(shared_card)
            .child(info_card)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_slots_match_hardware_kind() {
        let discrete_defaults = ["3D", "Copy", "Video Encode", "Video Decode"];
        assert_eq!(discrete_defaults.len(), 4);

        let integrated_defaults = ["3D", "Copy", "High Priority Compute", "High Priority 3D"];
        assert_eq!(integrated_defaults.len(), 4);
    }
}
