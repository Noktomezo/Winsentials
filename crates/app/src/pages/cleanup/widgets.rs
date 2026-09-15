use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ClickEvent, ElementId, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Rgba, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Transformation, Window, div, ease_in_out,
    linear_color_stop, linear_gradient, px, radians, svg,
};

use crate::entities::cleanup::format_bytes;
use crate::shared::motion::lerp_rgba;
use crate::shared::theme::Theme;
use crate::shared::ui::{Badge, BadgeVariant, Checkbox, Icon, TooltipHoverHandler};

pub type TargetHandler = Rc<dyn Fn(String, &mut Window, &mut App)>;
pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct TargetRow {
    pub id: String,
    pub name: String,
    pub secondary: String,
    pub bytes: u64,
    pub selected: bool,
}

pub fn badge(id: String, text: String, _theme: &Theme) -> AnyElement {
    Badge::new(id, text)
        .variant(BadgeVariant::Secondary)
        .into_any_element()
}

pub fn checkbox(
    id: String,
    checked: bool,
    enabled: bool,
    on_toggle: TargetHandler,
    tooltip: Option<impl Into<gpui::SharedString>>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
) -> AnyElement {
    let click_id = id.clone();
    let mut cb = Checkbox::new(format!("cb_{id}"))
        .checked(checked)
        .disabled(!enabled)
        .on_hover_tooltip_opt(on_hover_tooltip)
        .on_toggle(move |_new_checked, window, cx| {
            on_toggle(click_id.clone(), window, cx);
        });
    if let Some(tt) = tooltip {
        cb = cb.tooltip(tt);
    }
    cb.into_any_element()
}

pub fn clean_button(
    id: String,
    label: String,
    enabled: bool,
    reduce_motion: bool,
    theme: &Theme,
    on_click: Option<ClickHandler>,
) -> AnyElement {
    let theme = *theme;
    let target = if enabled { 1.0 } else { 0.0 };

    let mut base = div()
        .id(ElementId::Name(format!("{id}_btn").into()))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .h(px(32.0))
        .px(px(12.0))
        .rounded(px(6.0))
        .border_1()
        .text_size(px(13.0))
        .font_weight(FontWeight::MEDIUM)
        .flex_none();

    if enabled {
        base = base
            .cursor_pointer()
            .hover(|s| s.bg(theme.accent_blue.opacity(0.88)))
            .active(|s| s.bg(theme.accent_blue.opacity(0.75)));

        if let Some(handler) = on_click {
            base = base.on_click(move |event, window, cx| {
                cx.stop_propagation();
                handler(event, window, cx);
            });
        }
    }

    if reduce_motion {
        let (bg, border, text, opacity) = if enabled {
            (
                theme.accent_blue,
                theme.accent_blue,
                theme.selected_text,
                1.0,
            )
        } else {
            (theme.input_bg, theme.card_border, theme.text_muted, 0.45)
        };
        base.bg(bg)
            .border_color(border)
            .text_color(text)
            .opacity(opacity)
            .child(
                svg()
                    .path("icons/trash-2.svg")
                    .size(px(14.0))
                    .text_color(text)
                    .flex_none(),
            )
            .child(label)
            .into_any_element()
    } else {
        let input_bg = theme.input_bg;
        let accent_blue = theme.accent_blue;
        let card_border = theme.card_border;
        let text_muted = theme.text_muted;
        let selected_text = theme.selected_text;
        let spring = SpringAnimation::new(SpringConfig::new(260.0, 24.0, 1.0))
            .to(target)
            .with_epsilon(0.005);

        base.with_spring(
            ElementId::Name(format!("{id}_spring").into()),
            spring,
            move |btn, val| {
                let progress = val.clamp(0.0, 1.0);
                let current_bg = lerp_rgba(input_bg, accent_blue, progress);
                let current_border = lerp_rgba(card_border, accent_blue, progress);
                let current_text = lerp_rgba(text_muted, selected_text, progress);
                let current_opacity = 0.45 + 0.55 * progress;
                btn.bg(current_bg)
                    .border_color(current_border)
                    .text_color(current_text)
                    .opacity(current_opacity)
                    .child(
                        svg()
                            .path("icons/trash-2.svg")
                            .size(px(14.0))
                            .text_color(current_text)
                            .flex_none(),
                    )
                    .child(label.clone())
            },
        )
        .into_any_element()
    }
}

pub fn render_target(
    target: &TargetRow,
    icon: &'static str,
    accent_color: Rgba,
    theme: &Theme,
    on_toggle: TargetHandler,
    on_hover_tooltip: Option<TooltipHoverHandler>,
) -> AnyElement {
    let theme = *theme;
    let id = target.id.clone();
    let tooltip = if target.selected {
        rust_i18n::t!("cleanup.deselect_target")
    } else {
        rust_i18n::t!("cleanup.select_target")
    };
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .h(px(50.0))
        .px(px(10.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(theme.card_border)
        .bg(theme.main_bg)
        .child(checkbox(
            id,
            target.selected,
            true,
            on_toggle,
            Some(tooltip),
            on_hover_tooltip,
        ))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded(px(6.0))
                .bg(accent_color.opacity(0.12))
                .child(Icon::new(icon).size(px(13.0)).color(accent_color)),
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
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .text_ellipsis()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(target.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(target.secondary.clone()),
                ),
        )
        .child(
            div()
                .text_size(px(11.5))
                .text_color(theme.text_muted)
                .child(format_bytes(target.bytes)),
        )
        .into_any_element()
}

#[derive(Clone, Copy)]
pub struct CategoryChevronProps<'a> {
    pub category_id: &'static str,
    pub expanded: bool,
    pub has_targets: bool,
    pub cat_busy: bool,
    pub reduce_motion: bool,
    pub theme: &'a Theme,
}

pub fn category_chevron(props: CategoryChevronProps<'_>) -> AnyElement {
    let theme = props.theme;
    let chevron_color = if !props.has_targets {
        theme.text_muted.opacity(0.2)
    } else if props.cat_busy {
        theme.text_muted.opacity(0.35)
    } else {
        theme.text_muted
    };
    let chevron_el = if props.reduce_motion {
        let angle = if props.expanded {
            std::f32::consts::PI
        } else {
            0.0
        };
        svg()
            .path("icons/chevron-down.svg")
            .size(px(16.0))
            .text_color(chevron_color)
            .with_transformation(Transformation::rotate(radians(angle)))
            .into_any_element()
    } else {
        let target_angle = if props.expanded {
            std::f32::consts::PI
        } else {
            0.0
        };
        let category_id = props.category_id;
        svg()
            .path("icons/chevron-down.svg")
            .size(px(16.0))
            .text_color(chevron_color)
            .with_spring(
                ElementId::Name(format!("cleanup_chevron_{category_id}").into()),
                SpringAnimation::new(SpringConfig::new(300.0, 28.0, 1.0))
                    .to(target_angle)
                    .with_epsilon(0.01),
                |svg_el, angle| svg_el.with_transformation(Transformation::rotate(radians(angle))),
            )
            .into_any_element()
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .flex_none()
        .child(chevron_el)
        .into_any_element()
}

#[derive(Clone, Copy)]
pub struct CardProps<'a> {
    pub category_id: &'static str,
    pub idx: usize,
    pub scanning: bool,
    pub cleaning: bool,
    pub recently_cleaned: bool,
    pub selected: bool,
    pub reduce_motion: bool,
    pub theme: &'a Theme,
}

#[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
pub fn render_card(
    props: CardProps<'_>,
    header: impl IntoElement,
    body: impl IntoElement,
) -> AnyElement {
    let theme = *props.theme;
    let category_id = props.category_id;
    if (props.scanning || props.cleaning) && !props.reduce_motion {
        let is_clean = props.cleaning;
        let anim_id = if is_clean {
            format!("cleanup_clean_wave_{category_id}")
        } else {
            format!("cleanup_scan_wave_{category_id}")
        };
        let phase_offset = (props.idx as f32) * 0.9;
        let accent_color = if is_clean {
            theme.accent_orange
        } else {
            theme.accent_blue
        };
        let card_inner = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(9.0))
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body);

        div()
            .id(ElementId::Name(
                format!("cleanup_card_{category_id}").into(),
            ))
            .flex()
            .w_full()
            .rounded(px(10.0))
            .p(px(1.0))
            .overflow_hidden()
            .with_animation(
                ElementId::Name(anim_id.into()),
                Animation::new(Duration::from_millis(2400)).repeat(),
                move |wrap, delta| {
                    let phase = delta * std::f32::consts::TAU + phase_offset;
                    let angle = 90.0 + 35.0 * phase.sin();
                    let sweep = phase.sin() * 0.5 + 0.5;
                    let pulse = 0.70 + 0.30 * (phase * 2.0).sin();
                    let pos0 = -0.3 + sweep * 0.8;
                    let pos1 = pos0 + 0.8;
                    let wave_bg = linear_gradient(
                        angle,
                        linear_color_stop(accent_color.opacity(pulse), pos0),
                        linear_color_stop(theme.card_border.opacity(0.35), pos1),
                    );
                    wrap.bg(wave_bg)
                },
            )
            .child(card_inner)
            .into_any_element()
    } else if props.recently_cleaned && !props.reduce_motion {
        let anim_id = format!("cleanup_fade_{category_id}");
        let card_inner = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(9.0))
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body);

        div()
            .id(ElementId::Name(
                format!("cleanup_card_{category_id}").into(),
            ))
            .flex()
            .w_full()
            .rounded(px(10.0))
            .p(px(1.0))
            .overflow_hidden()
            .with_animation(
                ElementId::Name(anim_id.into()),
                Animation::new(Duration::from_millis(2000)).with_easing(ease_in_out),
                move |wrap, delta| {
                    let fade = (1.0 - delta).clamp(0.0, 1.0);
                    let border_color = if fade > 0.02 {
                        theme.accent_green.opacity(fade)
                    } else {
                        theme.card_border
                    };
                    wrap.bg(border_color)
                },
            )
            .child(card_inner)
            .into_any_element()
    } else {
        let card = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(10.0))
            .border_1()
            .bg(theme.card_bg)
            .overflow_hidden()
            .child(header)
            .child(body);

        if props.recently_cleaned {
            card.border_color(theme.accent_green).into_any_element()
        } else if props.cleaning {
            card.border_color(theme.accent_orange).into_any_element()
        } else if props.scanning {
            card.border_color(theme.accent_blue.opacity(0.6))
                .into_any_element()
        } else if props.reduce_motion {
            let border_color = if props.selected {
                theme.accent_blue
            } else {
                theme.card_border
            };
            card.border_color(border_color).into_any_element()
        } else {
            let base_border = theme.card_border;
            let active_border = theme.accent_blue;
            let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
                .to(if props.selected { 1.0 } else { 0.0 })
                .with_epsilon(0.005);
            card.with_spring(
                ElementId::Name(format!("cleanup_card_border_{category_id}").into()),
                spring,
                move |c, val| {
                    let progress = val.clamp(0.0, 1.0);
                    c.border_color(lerp_rgba(base_border, active_border, progress))
                },
            )
            .into_any_element()
        }
    }
}
