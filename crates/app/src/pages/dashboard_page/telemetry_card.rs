use std::sync::Arc;

use gpui::{
    AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Rgba, SharedString,
    SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Transformation, Window, div,
    point, px, svg,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;
use crate::widgets::sidebar::lerp_rgba;

pub type TelemetryCardHoverHandler =
    Arc<dyn Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static>;
pub type TelemetryCardClickHandler = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync + 'static>;

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
pub(crate) fn semantic_gpu_color(usage_pct: u32, temp_c: u32, theme: &Theme) -> Rgba {
    if usage_pct > 85 || temp_c > 80 {
        theme.accent_red
    } else if usage_pct >= 60 || temp_c >= 65 {
        theme.accent_yellow
    } else {
        theme.accent_green
    }
}

pub(crate) fn render_metric_label(text: impl Into<SharedString>, color: Rgba) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(14.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .text_ellipsis()
        .overflow_hidden()
        .whitespace_nowrap()
        .child(text.into())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn render_telemetry_card(
    card_id: &'static str,
    icon: impl Into<SharedString>,
    icon_color: Rgba,
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    metric_el: impl IntoElement,
    is_hovered: bool,
    theme: &Theme,
    on_hover: Option<TelemetryCardHoverHandler>,
    on_click: Option<TelemetryCardClickHandler>,
) -> impl IntoElement {
    let target = if is_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(320.0, 26.0, 1.0))
        .to(target)
        .with_epsilon(0.005);

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
        .child(Icon::new(icon).size(px(16.0)).color(icon_color));

    let text_stack = div()
        .flex()
        .flex_col()
        .justify_between()
        .h(px(32.0))
        .flex_1()
        .min_w(px(0.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .min_w(px(0.0))
                .child(
                    div()
                        .flex_none()
                        .text_size(px(13.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .whitespace_nowrap()
                        .child(title.into()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.5))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::NORMAL)
                        .text_color(theme.text_muted)
                        .text_ellipsis()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(detail.into()),
                ),
        )
        .child(div().flex().items_center().min_w(px(0.0)).child(metric_el));

    let text_muted = theme.text_muted;
    let text_primary = theme.text_primary;

    let chevron = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .with_spring(
            ElementId::Name(format!("{card_id}_chev_spring").into()),
            spring.clone(),
            move |chev, val| {
                let t = val.clamp(0.0, 1.0);
                let slide_x = t * 5.0;
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
        );

    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let input_border = theme.input_border;

    let on_hov = on_hover;
    let id_str: SharedString = card_id.into();

    let mut card_el = div()
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
            if let Some(ref h) = on_hov {
                h(id_str.clone(), hovered, window, cx);
            }
        });

    if let Some(on_clk) = on_click {
        card_el = card_el.on_click(move |_, window, cx| {
            on_clk(window, cx);
        });
    }

    card_el
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
        .child(icon_box)
        .child(text_stack)
        .child(chevron)
}