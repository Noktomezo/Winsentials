use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, ElementId, InteractiveElement, Interpolate, IntoElement,
    ParentElement, RenderOnce, Rgba, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, div, ease_in_out, px, svg,
};

use crate::shared::theme::Theme;
use crate::shared::ui::icon::Icon;

pub type BreadcrumbClickHandler = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type BreadcrumbHoverHandler = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

pub struct BreadcrumbItem {
    id: &'static str,
    label: SharedString,
    icon: Option<SharedString>,
    is_current: bool,
    is_hovered: bool,
    on_click: Option<BreadcrumbClickHandler>,
    on_hover: Option<BreadcrumbHoverHandler>,
}

impl BreadcrumbItem {
    #[must_use]
    pub fn new(id: &'static str, label: impl Into<SharedString>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            is_current: false,
            is_hovered: false,
            on_click: None,
            on_hover: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn current(mut self, is_current: bool) -> Self {
        self.is_current = is_current;
        self
    }

    #[must_use]
    pub fn hovered(mut self, is_hovered: bool) -> Self {
        self.is_hovered = is_hovered;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(handler));
        self
    }
}

#[derive(IntoElement)]
pub struct Breadcrumbs {
    items: Vec<BreadcrumbItem>,
    anim_key: SharedString,
}

impl Breadcrumbs {
    #[must_use]
    pub fn new(anim_key: impl Into<SharedString>) -> Self {
        Self {
            items: Vec::new(),
            anim_key: anim_key.into(),
        }
    }

    #[must_use]
    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn items(mut self, items: Vec<BreadcrumbItem>) -> Self {
        self.items = items;
        self
    }
}

impl RenderOnce for Breadcrumbs {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let items_len = self.items.len();
        let anim_key = self.anim_key;

        let mut row = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(6.0));

        for (idx, item) in self.items.into_iter().enumerate() {
            let is_last = idx + 1 == items_len;
            let is_current = item.is_current || is_last;
            let is_hovered = item.is_hovered;
            let on_click = item.on_click;
            let on_hover = item.on_hover;
            let label = item.label;

            let icon_el = item.icon.map(|icon_path| {
                Icon::new(icon_path).size(px(13.0)).color(if is_current {
                    theme.text_primary
                } else {
                    theme.text_muted
                })
            });

            let text_color = if is_current {
                theme.text_primary
            } else {
                theme.text_muted
            };

            let hover_color = theme.text_primary;
            let hover_spring = SpringAnimation::new(SpringConfig::new(350.0, 28.0, 1.0))
                .to(if is_hovered { 1.0 } else { 0.0 })
                .with_epsilon(0.005);

            let mut item_btn = div()
                .id(ElementId::Name(format!("breadcrumb_{}", item.id).into()))
                .debug_selector(|| format!("breadcrumb_{}", item.id))
                .flex()
                .items_center()
                .gap(px(5.0))
                .text_size(px(12.0))
                .font_weight(if is_current {
                    gpui::FontWeight::MEDIUM
                } else {
                    gpui::FontWeight::NORMAL
                })
                .text_color(text_color);

            if let Some(handler) = on_click {
                item_btn = item_btn.cursor_pointer().on_click(move |_, window, cx| {
                    handler(window, cx);
                });
            }

            if let Some(handler) = on_hover {
                item_btn = item_btn.on_hover(move |&hovered, window, cx| {
                    handler(hovered, window, cx);
                });
            }

            item_btn = item_btn.children(icon_el).child(label);
            row = row.child(item_btn.with_spring(
                ElementId::Name(format!("breadcrumb_{}_hover", item.id).into()),
                hover_spring,
                move |button, value| {
                    button.text_color(Rgba::interpolate(
                        text_color,
                        hover_color,
                        value.clamp(0.0, 1.0),
                    ))
                },
            ));

            if !is_last {
                let separator = svg()
                    .path("icons/chevron-right.svg")
                    .size(px(12.0))
                    .text_color(theme.text_muted);
                row = row.child(separator);
            }
        }

        // Smooth cross-fade transition when navigation / active route changes
        row.with_animation(
            ElementId::Name(format!("breadcrumbs_morph_{anim_key}").into()),
            Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
            gpui::Styled::opacity,
        )
    }
}
