use std::sync::Arc;

use gpui::{
    AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Rgba, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled,
    Transformation, Window, div, point, px, svg,
};

use crate::entities::tweaks::TweakSearchResult;
use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::lerp_rgba;

pub type TweakResultSelectHandler =
    Arc<dyn Fn(TweakSearchResult, &mut Window, &mut App) + Send + Sync + 'static>;
pub type TweakResultHoverHandler =
    Arc<dyn Fn(usize, bool, &mut Window, &mut App) + Send + Sync + 'static>;

#[derive(IntoElement)]
pub struct TweakResultCard {
    index: usize,
    result: TweakSearchResult,
    is_selected: bool,
    is_hovered: bool,
    on_select: Option<TweakResultSelectHandler>,
    on_hover: Option<TweakResultHoverHandler>,
}

impl TweakResultCard {
    #[must_use]
    pub fn new(
        index: usize,
        result: TweakSearchResult,
        is_selected: bool,
        is_hovered: bool,
    ) -> Self {
        Self {
            index,
            result,
            is_selected,
            is_hovered,
            on_select: None,
            on_hover: None,
        }
    }

    #[must_use]
    pub fn on_select(
        mut self,
        handler: impl Fn(TweakSearchResult, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover(
        mut self,
        handler: impl Fn(usize, bool, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_hover = Some(Arc::new(handler));
        self
    }
}

fn render_result_chevron(
    tweak_id: &'static str,
    spring: SpringAnimation<f32>,
    text_muted: Rgba,
    text_primary: Rgba,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .with_spring(
            ElementId::Name(format!("tweak_res_chev_{tweak_id}").into()),
            spring,
            move |chev, val| {
                let progress = val.clamp(0.0, 1.0);
                let slide_x = progress * 4.0;
                let col = lerp_rgba(text_muted, text_primary, progress);
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

impl RenderOnce for TweakResultCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let result = self.result;
        let index = self.index;
        let is_active = self.is_selected || self.is_hovered;
        let target_val = if is_active { 1.0 } else { 0.0 };

        let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
            .to(target_val)
            .with_epsilon(0.01);

        let chevron = render_result_chevron(
            result.tweak_id,
            spring.clone(),
            theme.text_muted,
            theme.text_primary,
        );

        let on_select = self.on_select;
        let on_hover = self.on_hover;
        let res_for_click = result.clone();

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
            .child(
                Icon::new(result.icon)
                    .size(px(16.0))
                    .color(theme.accent_blue),
            );

        let text_col = div()
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(2.0))
            .flex_1()
            .min_w(px(0.0))
            .overflow_hidden()
            .child(
                div()
                    .text_size(px(13.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_primary)
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(result.title),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .line_height(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(result.category_name),
            );

        let base_bg = theme.card_bg.opacity(0.0);
        let active_bg = theme.input_bg.opacity(0.45);
        let base_border = theme.card_border.opacity(0.0);
        let active_border = theme.accent_blue.opacity(0.4);

        div()
            .id(ElementId::Name(
                format!("tweak_res_card_{}", result.tweak_id).into(),
            ))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(10.0))
            .rounded(px(8.0))
            .border_1()
            .px(px(10.0))
            .py(px(8.0))
            .w_full()
            .on_hover(move |&hov, window, cx| {
                if let Some(ref h) = on_hover {
                    h(index, hov, window, cx);
                }
            })
            .on_click(move |_, window, cx| {
                if let Some(ref h) = on_select {
                    h(res_for_click.clone(), window, cx);
                }
            })
            .with_spring(
                ElementId::Name(format!("tweak_res_bg_{}", result.tweak_id).into()),
                spring,
                move |card, val| {
                    let progress = val.clamp(0.0, 1.0);
                    let bg = lerp_rgba(base_bg, active_bg, progress);
                    let border = lerp_rgba(base_border, active_border, progress);
                    card.bg(bg).border_color(border)
                },
            )
            .child(icon_box)
            .child(text_col)
            .child(chevron)
    }
}
