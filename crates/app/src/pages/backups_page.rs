use std::sync::Arc;

use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px,
};

use super::tools::backup_card::{BackupActionHandler, BackupHoverHandler, render_backup_card};
use crate::entities::tweaks::TweakBackup;
use crate::features::navigation::AppRoute;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::shared::ui::icon::Icon;

pub type CreateBackupHandler = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync + 'static>;

#[derive(IntoElement)]
pub struct BackupsPage {
    hovered_card: Option<SharedString>,
    backups: Vec<TweakBackup>,
    on_hover_card: Option<BackupHoverHandler>,
    on_create_backup: Option<CreateBackupHandler>,
    on_restore_backup: Option<BackupActionHandler>,
    on_rename_backup: Option<BackupActionHandler>,
    on_delete_backup: Option<BackupActionHandler>,
}

impl BackupsPage {
    #[must_use]
    pub fn new(hovered_card: Option<SharedString>) -> Self {
        Self {
            hovered_card,
            backups: Vec::new(),
            on_hover_card: None,
            on_create_backup: None,
            on_restore_backup: None,
            on_rename_backup: None,
            on_delete_backup: None,
        }
    }

    #[must_use]
    pub fn backups(mut self, backups: Vec<TweakBackup>) -> Self {
        self.backups = backups;
        self
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
    pub fn on_create_backup(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_create_backup = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_restore_backup(
        mut self,
        handler: impl Fn(String, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_restore_backup = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_rename_backup(
        mut self,
        handler: impl Fn(String, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_rename_backup = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_delete_backup(
        mut self,
        handler: impl Fn(String, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_delete_backup = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for BackupsPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Backups;
        let on_hover = self.on_hover_card;
        let hovered_card = self.hovered_card;

        let on_create = self.on_create_backup;
        let on_restore = self.on_restore_backup;
        let on_rename = self.on_rename_backup;
        let on_delete = self.on_delete_backup;

        let create_btn = Button::new(
            "create_backup_btn",
            rust_i18n::t!("tools.create_backup").to_string(),
        )
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Sm)
        .icon_left("icons/plus.svg")
        .on_click(move |_ev, window, cx| {
            if let Some(ref cb) = on_create {
                cb(window, cx);
            }
        });

        let backup_hover: Option<BackupHoverHandler> = on_hover;

        let backups_section: AnyElement = if self.backups.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .p(px(24.0))
                .gap(px(8.0))
                .rounded(px(10.0))
                .border_1()
                .border_color(theme.card_border)
                .bg(theme.card_bg)
                .child(
                    Icon::new("icons/archive.svg")
                        .size(px(24.0))
                        .color(theme.text_muted),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_primary)
                        .child(rust_i18n::t!("tools.no_backups").to_string()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(theme.text_muted)
                        .child(rust_i18n::t!("tools.no_backups_desc").to_string()),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(self.backups.iter().map(|b| {
                    let card_id = format!("backup_card_{}", b.id);
                    let is_hovered = hovered_card
                        .as_ref()
                        .is_some_and(|id| id.as_ref() == card_id);
                    render_backup_card(
                        b,
                        &theme,
                        is_hovered,
                        backup_hover.as_ref(),
                        on_restore.as_ref(),
                        on_rename.as_ref(),
                        on_delete.as_ref(),
                    )
                }))
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .p(px(16.0))
            .w_full()
            .child(PageHeader::new(route.title(), route.description()).actions(create_btn))
            .child(backups_section)
    }
}
