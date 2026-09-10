use std::sync::Arc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, SharedString, SpringAnimation, SpringConfig,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::entities::tweaks::TweakBackup;
use crate::shared::theme::Theme;
use crate::shared::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::ui::icon::Icon;
use crate::shared::ui::icon_button::{IconButton, IconButtonVariant};
use crate::widgets::sidebar::lerp_rgba;

pub type BackupHoverHandler =
    Arc<dyn Fn(SharedString, bool, &mut Window, &mut App) + Send + Sync + 'static>;
pub type BackupActionHandler = Arc<dyn Fn(String, &mut Window, &mut App) + Send + Sync + 'static>;

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn render_backup_card(
    backup: &TweakBackup,
    theme: &Theme,
    is_hovered: bool,
    on_hover: Option<&BackupHoverHandler>,
    on_restore: Option<&BackupActionHandler>,
    on_rename: Option<&BackupActionHandler>,
    on_delete: Option<&BackupActionHandler>,
) -> AnyElement {
    let backup_id = backup.id.clone();
    let card_id = format!("backup_card_{backup_id}");
    let id_shared: SharedString = card_id.clone().into();

    let target_val = if is_hovered { 1.0 } else { 0.0 };
    let spring = SpringAnimation::new(SpringConfig::new(260.0, 26.0, 1.0))
        .to(target_val)
        .with_epsilon(0.01);

    let card_bg = theme.card_bg;
    let input_bg = theme.input_bg;
    let card_border = theme.card_border;
    let input_border = theme.input_border;

    let on_hover_cb = on_hover.cloned();

    let restore_id = backup_id.clone();
    let rename_id = backup_id.clone();
    let delete_id = backup_id;

    let on_restore_cb = on_restore.cloned();
    let on_rename_cb = on_rename.cloned();
    let on_delete_cb = on_delete.cloned();

    let active = backup.active_count();
    let total = backup.total_count();
    let subtitle =
        rust_i18n::t!("tools.backup_active_tweaks", active = active, total = total).to_string();

    let title_line = format!("{} • {}", backup.name, backup.created_at);

    div()
        .id(ElementId::Name(format!("{card_id}_root").into()))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .rounded(px(10.0))
        .border_1()
        .p(px(16.0))
        .min_h(px(64.0))
        .w_full()
        .on_hover(move |&hovered, window, cx| {
            if let Some(ref h) = on_hover_cb {
                h(id_shared.clone(), hovered, window, cx);
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
                .gap(px(12.0))
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
                        .child(
                            Icon::new("icons/archive.svg")
                                .size(px(16.0))
                                .color(theme.accent_blue),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .gap(px(2.0))
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
                                .child(title_line),
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
                                .child(subtitle),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .flex_none()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .child(
                    Button::new(
                        format!("restore_{restore_id}"),
                        rust_i18n::t!("tools.restore_backup").to_string(),
                    )
                    .size(ButtonSize::Md)
                    .variant(ButtonVariant::Primary)
                    .icon_left("icons/archive-restore.svg")
                    .on_click(move |_ev, window, cx| {
                        if let Some(ref cb) = on_restore_cb {
                            cb(restore_id.clone(), window, cx);
                        }
                    }),
                )
                .child(
                    IconButton::new(format!("rename_{rename_id}"), "icons/pencil.svg")
                        .variant(IconButtonVariant::Outline)
                        .tooltip(rust_i18n::t!("tools.rename_backup").to_string())
                        .on_click(move |_ev, window, cx| {
                            if let Some(ref cb) = on_rename_cb {
                                cb(rename_id.clone(), window, cx);
                            }
                        }),
                )
                .child(
                    IconButton::new(format!("delete_{delete_id}"), "icons/trash-2.svg")
                        .variant(IconButtonVariant::Outline)
                        .destructive(true)
                        .tooltip(rust_i18n::t!("tools.delete_backup").to_string())
                        .on_click(move |_ev, window, cx| {
                            if let Some(ref cb) = on_delete_cb {
                                cb(delete_id.clone(), window, cx);
                            }
                        }),
                ),
        )
        .into_any_element()
}
