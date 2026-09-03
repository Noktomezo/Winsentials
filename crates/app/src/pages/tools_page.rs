use std::sync::Arc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Transformation, Window, div, point, px, svg,
};

use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
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
    pub route: AppRoute,
}

pub const SYSTEM_TOOLS: [ToolItem; 2] = [
    ToolItem {
        id: "startup",
        icon: "icons/rocket.svg",
        title_key: "tools.startup_title",
        desc_key: "tools.startup_desc",
        route: AppRoute::Startup,
    },
    ToolItem {
        id: "cleanup",
        icon: "icons/broom.svg",
        title_key: "cleanup.title",
        desc_key: "cleanup.desc",
        route: AppRoute::Cleanup,
    },
];

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

#[allow(clippy::too_many_lines)]
fn render_tool_card(
    tool: ToolItem,
    theme: &Theme,
    is_hovered: bool,
    on_hover: Option<ToolHoverHandler>,
    on_nav: Option<ToolNavigateHandler>,
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
    let input_border = theme.input_border;

    let id_str: SharedString = card_id.into();

    div()
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
            if let Some(ref h) = on_hover {
                h(id_str.clone(), hovered, window, cx);
            }
        })
        .on_click(move |_ev, window, cx| {
            if let Some(ref nav) = on_nav {
                nav(tool.route, window, cx);
            }
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
                .gap(px(10.0))
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(32.0))
                        .rounded(px(6.0))
                        .bg(theme.input_bg)
                        .border_1()
                        .border_color(theme.card_border)
                        .flex_none()
                        .child(Icon::new(tool.icon).size(px(16.0)).color(theme.accent_blue)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .h(px(32.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .line_height(px(16.0))
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
                                .line_height(px(14.0))
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

pub type ToolNavigateHandler = Arc<dyn Fn(AppRoute, &mut Window, &mut App) + Send + Sync + 'static>;

#[derive(IntoElement)]
pub struct ToolsPage {
    hovered_card: Option<SharedString>,
    on_hover_card: Option<ToolHoverHandler>,
    on_navigate: Option<ToolNavigateHandler>,
}

impl ToolsPage {
    #[must_use]
    pub fn new(hovered_card: Option<SharedString>) -> Self {
        Self {
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

impl RenderOnce for ToolsPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Tools;
        let on_hover = self.on_hover_card;
        let on_nav = self.on_navigate;
        let hovered_card = self.hovered_card;

        let card_elements: Vec<(&'static str, AnyElement)> = SYSTEM_TOOLS
            .iter()
            .map(|tool| {
                let tool = *tool;
                let is_hovered = hovered_card.as_ref().is_some_and(|id| id == tool.id);
                let elem =
                    render_tool_card(tool, &theme, is_hovered, on_hover.clone(), on_nav.clone());
                (tool.id, elem)
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .child(PageHeader::new(route.title(), route.description()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(card_elements.into_iter().map(|(_, card)| card)),
            )
    }
}
