use std::rc::Rc;
use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnimationExt, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SpringAnimation, SpringConfig, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::entities::cleanup::{CleanupCategory, CleanupState, format_bytes};
use crate::pages::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::smooth_scroll::SmoothVirtualList;
use crate::shared::ui::{
    Badge, BadgeVariant, Checkbox, Icon, IconButton, IconButtonVariant, TooltipHoverHandler,
    TooltipState,
};

#[path = "cleanup/widgets.rs"]
mod widgets;
use widgets::{
    CardProps, CategoryChevronProps, TargetHandler, TargetRow, badge, category_chevron,
    clean_button, render_card, render_target,
};

const TARGET_HEIGHT: f32 = 50.0;
const TARGET_GAP: f32 = 6.0;
const MAX_VISIBLE_TARGETS: usize = 6;

pub type CategoryHandler = Rc<dyn Fn(CleanupCategory, &mut Window, &mut App)>;
pub type SimpleHandler = Rc<dyn Fn(&mut Window, &mut App)>;
pub type CleanHandler = Rc<dyn Fn(Option<CleanupCategory>, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct CleanupPage {
    state: CleanupState,
    on_toggle_target: TargetHandler,
    on_toggle_category: CategoryHandler,
    on_toggle_expanded: CategoryHandler,
    on_toggle_all: SimpleHandler,
    on_refresh: SimpleHandler,
    on_clean: CleanHandler,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl CleanupPage {
    #[must_use]
    pub fn new(
        state: CleanupState,
        on_toggle_target: TargetHandler,
        on_toggle_category: CategoryHandler,
        on_toggle_expanded: CategoryHandler,
        on_toggle_all: SimpleHandler,
        on_refresh: SimpleHandler,
        on_clean: CleanHandler,
    ) -> Self {
        Self {
            state,
            on_toggle_target,
            on_toggle_category,
            on_toggle_expanded,
            on_toggle_all,
            on_refresh,
            on_clean,
            on_hover_tooltip: None,
        }
    }

    #[must_use]
    pub fn on_hover_tooltip(
        mut self,
        handler: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_tooltip_opt(mut self, handler: Option<TooltipHoverHandler>) -> Self {
        self.on_hover_tooltip = handler;
        self
    }
}

const fn target_list_id(category: CleanupCategory) -> &'static str {
    match category {
        CleanupCategory::Windows => "cleanup_windows_targets",
        CleanupCategory::Browsers => "cleanup_browsers_targets",
        CleanupCategory::Applications => "cleanup_applications_targets",
        CleanupCategory::Development => "cleanup_development_targets",
        CleanupCategory::Games => "cleanup_games_targets",
        CleanupCategory::Media => "cleanup_media_targets",
        CleanupCategory::Devices => "cleanup_devices_targets",
    }
}

impl RenderOnce for CleanupPage {
    #[allow(clippy::too_many_lines)]
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let reduce_motion = cx.reduce_motion();
        let (selected_count, _) = self.state.selected_totals();
        let is_scanning = self.state.scanning;
        let is_cleaning = self.state.cleaning;
        let busy = is_scanning || is_cleaning;
        let total = self.state.snapshot.targets.len();
        let total_bytes = self
            .state
            .snapshot
            .targets
            .iter()
            .map(|target| target.bytes)
            .sum();

        let clean_all = self.on_clean.clone();
        let toggle_all = self.on_toggle_all.clone();
        let refresh = self.on_refresh.clone();

        let on_hover_tooltip = self.on_hover_tooltip.clone();
        let all_selected = total > 0 && selected_count == total;
        let (check_all_icon, check_all_tooltip) = if all_selected {
            ("icons/square-x.svg", rust_i18n::t!("cleanup.uncheck_all"))
        } else {
            (
                "icons/square-check-big.svg",
                rust_i18n::t!("cleanup.check_all"),
            )
        };

        let header = PageHeader::new(
            rust_i18n::t!("cleanup.title"),
            rust_i18n::t!("cleanup.desc"),
        )
        .badge(if is_scanning {
            div().flex().items_center().gap(px(6.0)).child(
                Badge::new("cleanup_count", rust_i18n::t!("cleanup.scanning"))
                    .variant(BadgeVariant::Accent)
                    .loading(true),
            )
        } else {
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(badge(
                    "cleanup_count".into(),
                    rust_i18n::t!("cleanup.targets", count = total).to_string(),
                    &theme,
                ))
                .child(badge(
                    "cleanup_size".into(),
                    format_bytes(total_bytes),
                    &theme,
                ))
        })
        .actions(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(clean_button(
                    "cleanup_clean_all".into(),
                    rust_i18n::t!("cleanup.clean_all").to_string(),
                    selected_count > 0 && !busy,
                    reduce_motion,
                    &theme,
                    Some(Rc::new(move |_event, window, cx| {
                        clean_all(None, window, cx);
                    })),
                ))
                .child(
                    IconButton::new("cleanup_check_all", check_all_icon)
                        .variant(IconButtonVariant::Outline)
                        .selected(all_selected)
                        .tooltip(check_all_tooltip)
                        .on_hover_tooltip_opt(on_hover_tooltip.clone())
                        .disabled(total == 0 || busy)
                        .on_click(move |_event, window, cx| {
                            toggle_all(window, cx);
                        }),
                )
                .child(
                    IconButton::new("cleanup_refresh", "icons/refresh-cw.svg")
                        .variant(IconButtonVariant::Outline)
                        .tooltip(rust_i18n::t!("cleanup.refresh"))
                        .on_hover_tooltip_opt(on_hover_tooltip.clone())
                        .disabled(busy)
                        .loading(is_scanning)
                        .on_click(move |_event, window, cx| {
                            refresh(window, cx);
                        }),
                ),
        );

        let mut categories = div().flex().flex_col().gap(px(10.0));
        for (idx, &category) in CleanupCategory::ALL.iter().enumerate() {
            let targets = self
                .state
                .snapshot
                .targets
                .iter()
                .filter(|target| target.category == category)
                .collect::<Vec<_>>();
            let has_targets = !targets.is_empty();
            let checked = targets
                .iter()
                .filter(|target| self.state.selected.contains(&target.id))
                .count();
            let bytes = targets.iter().map(|target| target.bytes).sum::<u64>();
            let expanded = self.state.expanded == Some(category) && has_targets;
            let visible_count = targets.len().min(MAX_VISIBLE_TARGETS);
            let visible_count_f32 = f32::from(u16::try_from(visible_count).unwrap_or(u16::MAX));
            let list_height = if visible_count == 0 {
                0.0
            } else {
                32.0 + visible_count_f32 * TARGET_HEIGHT
                    + f32::from(u16::try_from(visible_count.saturating_sub(1)).unwrap_or(u16::MAX))
                        * TARGET_GAP
            };
            let full_height = if visible_count == 0 {
                0.0
            } else {
                list_height + 1.0
            };
            let all_checked = has_targets && checked == targets.len();
            let some_checked = has_targets && checked > 0 && checked < targets.len();
            let toggle_category = self.on_toggle_category.clone();
            let toggle_category_checkbox = self.on_toggle_category.clone();
            let toggle_expanded = self.on_toggle_expanded.clone();
            let clean_category = self.on_clean.clone();
            let cat_scanning = self.state.is_category_scanning(category);
            let cat_cleaning = self.state.is_category_cleaning(category);
            let cat_recently_cleaned = self.state.is_category_recently_cleaned(category);
            let cat_busy = cat_scanning || cat_cleaning;
            let category_id = category.id();
            let can_expand = !cat_busy && has_targets;

            let cat_checkbox_tooltip = if all_checked {
                rust_i18n::t!("cleanup.deselect_category")
            } else {
                rust_i18n::t!("cleanup.select_category")
            };
            let cat_check_icon = if all_checked {
                "icons/square-x.svg"
            } else {
                "icons/square-check-big.svg"
            };

            let header = div()
                .id(ElementId::Name(format!("cleanup_{category_id}").into()))
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(64.0))
                .px(px(16.0))
                .when(can_expand, |this| {
                    this.cursor_pointer().on_click(move |_event, window, cx| {
                        toggle_expanded(category, window, cx);
                    })
                })
                .child(
                    Checkbox::new(format!("cleanup_category_{category_id}"))
                        .checked(all_checked)
                        .indeterminate(some_checked)
                        .disabled(!has_targets || cat_busy)
                        .tooltip(cat_checkbox_tooltip.clone())
                        .on_hover_tooltip_opt(on_hover_tooltip.clone())
                        .on_toggle(move |_new_checked, window, cx| {
                            toggle_category_checkbox(category, window, cx);
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(32.0))
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(theme.card_border)
                        .bg(theme.input_bg)
                        .child(
                            Icon::new(category.icon())
                                .size(px(16.0))
                                .color(category.accent_color(&theme)),
                        ),
                )
                .child({
                    let secondary_text = if cat_scanning {
                        rust_i18n::t!("cleanup.scanning").to_string()
                    } else if category == CleanupCategory::Devices {
                        format!("{checked}/{}", targets.len())
                    } else {
                        format!("{checked}/{} • {}", targets.len(), format_bytes(bytes))
                    };
                    let secondary_color = if cat_scanning {
                        theme.accent_blue
                    } else {
                        theme.text_muted
                    };

                    div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .h(px(32.0))
                        .flex_1()
                        .min_w(px(0.0))
                        .child(
                            div()
                                .text_size(px(13.5))
                                .line_height(px(16.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_primary)
                                .child(rust_i18n::t!(format!("cleanup.category.{category_id}"))),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .line_height(px(14.0))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(secondary_color)
                                .child(secondary_text),
                        )
                })
                .child(category_chevron(CategoryChevronProps {
                    category_id,
                    expanded,
                    has_targets,
                    cat_busy,
                    reduce_motion,
                    theme: &theme,
                }))
                .child(clean_button(
                    format!("cleanup_{category_id}_clean"),
                    rust_i18n::t!("cleanup.clean").to_string(),
                    checked > 0 && !cat_busy,
                    reduce_motion,
                    &theme,
                    Some(Rc::new(move |_event, window, cx| {
                        clean_category(Some(category), window, cx);
                    })),
                ))
                .child(
                    IconButton::new(format!("cleanup_{category_id}_check"), cat_check_icon)
                        .variant(IconButtonVariant::Outline)
                        .selected(all_checked)
                        .tooltip(cat_checkbox_tooltip)
                        .on_hover_tooltip_opt(on_hover_tooltip.clone())
                        .disabled(!has_targets || cat_busy)
                        .on_click(move |_event, window, cx| {
                            cx.stop_propagation();
                            toggle_category(category, window, cx);
                        }),
                );

            let body = if has_targets {
                let rows = Arc::new(
                    targets
                        .iter()
                        .map(|target| TargetRow {
                            id: target.id.clone(),
                            name: target.name.clone(),
                            secondary: target.device_instance_id.clone().unwrap_or_else(|| {
                                rust_i18n::t!("cleanup.found_paths", count = target.paths.len())
                                    .to_string()
                            }),
                            bytes: target.bytes,
                            selected: self.state.selected.contains(&target.id),
                        })
                        .collect::<Vec<_>>(),
                );
                let rows_for_list = rows.clone();
                let target_toggle = self.on_toggle_target.clone();
                let cat_icon = category.icon();
                let cat_accent = category.accent_color(&theme);
                let target_hover_tooltip = on_hover_tooltip.clone();
                let list = SmoothVirtualList::new(
                    target_list_id(category),
                    rows.len(),
                    px(TARGET_HEIGHT),
                    px(TARGET_GAP),
                    move |index, _window, cx| {
                        let theme = Theme::get(cx);
                        render_target(
                            &rows_for_list[index],
                            cat_icon,
                            cat_accent,
                            &theme,
                            target_toggle.clone(),
                            target_hover_tooltip.clone(),
                        )
                    },
                );

                let target_height = if expanded { full_height } else { 0.0 };

                let body_container = div()
                    .id(ElementId::Name(
                        format!("cleanup_{category_id}_targets").into(),
                    ))
                    .flex()
                    .flex_col()
                    .min_h(px(0.0))
                    .flex_none()
                    .overflow_hidden()
                    .child(div().h(px(1.0)).mx(px(16.0)).bg(theme.card_border))
                    .child(div().h(px(list_height)).w_full().min_h(px(0.0)).child(list));

                if reduce_motion {
                    body_container.h(px(target_height)).into_any_element()
                } else {
                    body_container
                        .with_spring(
                            ElementId::Name(format!("cleanup_{category_id}_expand").into()),
                            SpringAnimation::new(SpringConfig::new(320.0, 28.0, 1.0))
                                .to(target_height)
                                .with_epsilon(0.5),
                            move |body, height| {
                                let opacity = if full_height > 0.0 {
                                    (height / full_height).clamp(0.0, 1.0)
                                } else {
                                    0.0
                                };
                                body.h(px(height)).opacity(opacity)
                            },
                        )
                        .into_any_element()
                }
            } else {
                div().into_any_element()
            };

            let card = render_card(
                CardProps {
                    category_id,
                    idx,
                    scanning: cat_scanning,
                    cleaning: cat_cleaning,
                    recently_cleaned: cat_recently_cleaned,
                    selected: checked > 0,
                    reduce_motion,
                    theme: &theme,
                },
                header,
                body,
            );
            categories = categories.child(card);
        }

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .child(header)
            .child(categories)
    }
}
