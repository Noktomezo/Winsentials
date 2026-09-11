use std::rc::Rc;

use gpui::{Context, SharedString};

use super::AppView;
use crate::entities::cleanup::CleanupCategory;
use crate::features::navigation::AppRoute;
use crate::pages::{BackupsPage, CleanupPage, ToolsPage};

impl AppView {
    pub(super) fn build_cleanup_page(
        &self,
        current_route: AppRoute,
        cx: &mut Context<Self>,
    ) -> Option<CleanupPage> {
        if current_route != AppRoute::Cleanup {
            return None;
        }

        let on_cleanup_toggle_target = cx.listener(|this, id: &String, _window, cx| {
            this.cleanup.toggle_target(id);
            cx.notify();
        });
        let on_cleanup_toggle_category =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                this.cleanup.toggle_category(*category);
                cx.notify();
            });
        let on_cleanup_toggle_expanded =
            cx.listener(|this, category: &CleanupCategory, _window, cx| {
                let has_targets = this
                    .cleanup
                    .snapshot
                    .targets
                    .iter()
                    .any(|t| t.category == *category);
                if !has_targets {
                    return;
                }
                this.cleanup.expanded =
                    (this.cleanup.expanded != Some(*category)).then_some(*category);
                cx.notify();
            });
        let on_cleanup_toggle_all = cx.listener(|this, _event: &(), _window, cx| {
            this.cleanup.toggle_all();
            cx.notify();
        });
        let on_cleanup_refresh = cx.listener(|this, _event: &(), _window, cx| {
            this.refresh_cleanup(cx);
        });
        let on_cleanup_clean =
            cx.listener(|this, category: &Option<CleanupCategory>, _window, cx| {
                this.clean_cleanup(*category, cx);
            });

        Some(CleanupPage::new(
            self.cleanup.clone(),
            Rc::new(move |id, window, cx| {
                on_cleanup_toggle_target(&id, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_category(&category, window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_toggle_expanded(&category, window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_toggle_all(&(), window, cx);
            }),
            Rc::new(move |window, cx| {
                on_cleanup_refresh(&(), window, cx);
            }),
            Rc::new(move |category, window, cx| {
                on_cleanup_clean(&category, window, cx);
            }),
        ))
    }

    pub(super) fn build_tools_page(
        current_route: AppRoute,
        hovered_telemetry_card: Option<SharedString>,
        cx: &mut Context<Self>,
    ) -> Option<ToolsPage> {
        if current_route != AppRoute::Tools {
            return None;
        }

        let on_nav = cx.listener(|this, route: &AppRoute, window, cx| {
            this.navigate_to(*route, window, cx);
        });
        let on_hover = cx.listener(
            |this, &(ref card_id, is_hovered): &(SharedString, bool), window, cx| {
                this.set_hovered_telemetry_card(card_id.clone(), is_hovered, window, cx);
            },
        );

        Some(
            ToolsPage::new(hovered_telemetry_card)
                .on_navigate(move |r, w, cx| on_nav(&r, w, cx))
                .on_hover_card(move |id, val, w, cx| on_hover(&(id, val), w, cx)),
        )
    }

    pub(super) fn build_backups_page(
        &self,
        current_route: AppRoute,
        hovered_telemetry_card: Option<SharedString>,
        cx: &mut Context<Self>,
    ) -> Option<BackupsPage> {
        if current_route != AppRoute::Backups {
            return None;
        }

        let on_hover = cx.listener(
            |this, &(ref card_id, is_hovered): &(SharedString, bool), window, cx| {
                this.set_hovered_telemetry_card(card_id.clone(), is_hovered, window, cx);
            },
        );
        let on_create =
            cx.listener(|this, _event: &(), _window, cx| this.open_create_backup_modal(cx));
        let on_restore = cx.listener(|this, id: &String, _window, cx| {
            this.confirm_restore_backup(id.clone(), cx);
        });
        let on_rename = cx.listener(|this, id: &String, _window, cx| {
            this.open_rename_backup_modal(id.clone(), cx);
        });
        let on_delete = cx.listener(|this, id: &String, _window, cx| {
            this.confirm_delete_backup(id.clone(), cx);
        });

        Some(
            BackupsPage::new(hovered_telemetry_card)
                .backups(self.tweak_backups.clone())
                .on_hover_card(move |id, val, w, cx| on_hover(&(id, val), w, cx))
                .on_create_backup(move |w, cx| on_create(&(), w, cx))
                .on_restore_backup(move |id, w, cx| on_restore(&id, w, cx))
                .on_rename_backup(move |id, w, cx| on_rename(&id, w, cx))
                .on_delete_backup(move |id, w, cx| on_delete(&id, w, cx)),
        )
    }
}
