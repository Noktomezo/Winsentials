use std::sync::Arc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Transformation, Window, div, point, px, svg,
};

use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::animated_grid::render_animated_grid;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::lerp_rgba;

pub type ToolHoverHandler =
    Arc<dyn Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug)]
pub struct ToolItem {
    pub id: &'static str,
    pub icon: &'static str,
    pub title_key: &'static str,
    pub desc_key: &'static str,
    pub command: &'static str,
}

pub const SYSTEM_TOOLS: [ToolItem; 11] = [
    ToolItem {
        id: "startup",
        icon: "icons/rocket.svg",
        title_key: "tools.startup_title",
        desc_key: "tools.startup_desc",
        command: "ms-settings:startupapps",
    },
    ToolItem {
        id: "cleanmgr",
        icon: "icons/trash-2.svg",
        title_key: "tools.cleanmgr_title",
        desc_key: "tools.cleanmgr_desc",
        command: "cleanmgr.exe",
    },
    ToolItem {
        id: "taskmgr",
        icon: "icons/activity.svg",
        title_key: "tools.taskmgr_title",
        desc_key: "tools.taskmgr_desc",
        command: "taskmgr.exe",
    },
    ToolItem {
        id: "resmon",
        icon: "icons/gauge.svg",
        title_key: "tools.resmon_title",
        desc_key: "tools.resmon_desc",
        command: "resmon.exe",
    },
    ToolItem {
        id: "regedit",
        icon: "icons/binary.svg",
        title_key: "tools.regedit_title",
        desc_key: "tools.regedit_desc",
        command: "regedit.exe",
    },
    ToolItem {
        id: "services",
        icon: "icons/cog.svg",
        title_key: "tools.services_title",
        desc_key: "tools.services_desc",
        command: "services.msc",
    },
    ToolItem {
        id: "devmgmt",
        icon: "icons/cpu.svg",
        title_key: "tools.devmgmt_title",
        desc_key: "tools.devmgmt_desc",
        command: "devmgmt.msc",
    },
    ToolItem {
        id: "compmgmt",
        icon: "icons/monitor.svg",
        title_key: "tools.compmgmt_title",
        desc_key: "tools.compmgmt_desc",
        command: "compmgmt.msc",
    },
    ToolItem {
        id: "ncpa",
        icon: "icons/network.svg",
        title_key: "tools.ncpa_title",
        desc_key: "tools.ncpa_desc",
        command: "ncpa.cpl",
    },
    ToolItem {
        id: "power",
        icon: "icons/zap.svg",
        title_key: "tools.power_title",
        desc_key: "tools.power_desc",
        command: "powercfg.cpl",
    },
    ToolItem {
        id: "dxdiag",
        icon: "icons/gamepad.svg",
        title_key: "tools.dxdiag_title",
        desc_key: "tools.dxdiag_desc",
        command: "dxdiag.exe",
    },
];

pub fn launch_system_tool(cmd: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", cmd])
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

fn render_tool_chevron(
    card_id: &'static str,
    spring: SpringAnimation<f32>,
    text_muted: gpui::Rgba,
    text_primary: gpui::Rgba,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .with_spring(
            ElementId::Name(format!("{card_id}_chev_spring").into()),
            spring,
            move |chev, val| {
                let t = val.clamp(0.0, 1.0);
                let slide_x = t * 4.0;
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
        )
}

fn render_tool_card(
    tool: ToolItem,
    theme: &Theme,
    is_hovered: bool,
    on_hover: Option<ToolHoverHandler>,
) -> AnyElement {
    let title = rust_i18n::t!(tool.title_key).to_string();
    let desc = rust_i18n::t!(tool.desc_key).to_string();
    let card_id = tool.id;

    let target_val = if is_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
        .to(target_val)
        .with_epsilon(0.01);

    let chevron = render_tool_chevron(
        card_id,
        spring.clone(),
        theme.text_muted,
        theme.text_primary,
    );

    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let input_border = theme.accent_blue;

    let id_str: SharedString = card_id.into();

    div()
        .id(ElementId::Name(format!("{card_id}_root").into()))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .rounded(px(10.0))
        .border_1()
        .p(px(14.0))
        .h(px(68.0))
        .w_full()
        .on_hover(move |&hovered, window, cx| {
            if let Some(ref h) = on_hover {
                h(id_str.clone(), hovered, window, cx);
            }
        })
        .on_click(move |_ev, _window, _cx| {
            launch_system_tool(tool.command);
        })
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
                .gap(px(12.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(36.0))
                        .rounded_lg()
                        .bg(theme.accent_blue.opacity(0.12))
                        .flex_none()
                        .child(Icon::new(tool.icon).size(px(20.0)).color(theme.accent_blue)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_primary)
                                .text_ellipsis()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .line_height(px(15.0))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(theme.text_muted)
                                .text_ellipsis()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(desc),
                        ),
                ),
        )
        .child(chevron)
        .into_any_element()
}

#[derive(IntoElement)]
pub struct ToolsPage {
    hovered_card: Option<SharedString>,
    sidebar_expanded: bool,
    on_hover_card: Option<ToolHoverHandler>,
}

impl ToolsPage {
    #[must_use]
    pub fn new(hovered_card: Option<SharedString>, sidebar_expanded: bool) -> Self {
        Self {
            hovered_card,
            sidebar_expanded,
            on_hover_card: None,
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
}

impl RenderOnce for ToolsPage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Tools;
        let on_hover = self.on_hover_card;
        let hovered_card = self.hovered_card;

        let window_w = window.viewport_size().width;
        let sidebar_w = if self.sidebar_expanded {
            px(200.0)
        } else {
            px(40.0)
        };
        let available_w = (window_w - sidebar_w - px(64.0)).max(px(300.0));

        let card_elements: Vec<(&'static str, AnyElement)> = SYSTEM_TOOLS
            .iter()
            .map(|tool| {
                let tool = *tool;
                let is_hovered = hovered_card.as_ref().is_some_and(|id| id == tool.id);
                let elem = render_tool_card(tool, &theme, is_hovered, on_hover.clone());
                (tool.id, elem)
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .p(px(24.0))
            .w_full()
            .child(PageHeader::new(route.title(), route.description()))
            .child(render_animated_grid(
                "tools_grid",
                available_w,
                px(340.0),
                px(68.0),
                px(10.0),
                card_elements,
            ))
    }
}
