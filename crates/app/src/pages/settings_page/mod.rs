use std::sync::Arc;

use gpui::{
    App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px,
};

use crate::features::discord_rpc::DiscordRpcActivity;
use crate::features::navigation::AppRoute;
use crate::features::updater::UpdateState;
use crate::pages::page_header::PageHeader;
use crate::shared::theme::Theme;
use crate::shared::ui::TooltipState;

pub mod appearance_card;
pub mod behavior_card;
pub mod types;
pub mod updates_card;

pub(crate) use appearance_card::*;
pub(crate) use behavior_card::*;
pub use types::*;
pub(crate) use updates_card::*;
#[allow(clippy::struct_excessive_bools)]
#[derive(IntoElement)]
pub struct SettingsPage {
    current_locale: &'static str,
    minimize_to_tray: bool,
    autostart: bool,
    autostart_to_tray: bool,
    discord_rpc: DiscordRpcActivity,
    check_updates: bool,
    update_state: UpdateState,
    open_dropdown: Option<&'static str>,
    open_dropdown_upward: bool,
    opening_dropdown: Option<&'static str>,
    closing_dropdown: Option<&'static str>,
    hovered_dropdown: Option<&'static str>,
    hovered_option: Option<(&'static str, &'static str)>,
    pending_selection: Option<(&'static str, &'static str)>,
    on_change_palette: Option<StringHandler>,
    on_change_language: Option<StringHandler>,
    on_change_theme: Option<StringHandler>,
    on_change_transparency: Option<BoolHandler>,
    on_toggle_minimize_to_tray: Option<BoolHandler>,
    on_toggle_autostart: Option<BoolHandler>,
    on_toggle_autostart_to_tray: Option<BoolHandler>,
    on_change_discord_rpc: Option<StringHandler>,
    on_toggle_check_updates: Option<BoolHandler>,
    on_check_update: Option<VoidHandler>,
    on_download_and_install_update: Option<VoidHandler>,
    on_toggle_dropdown: Option<DropdownToggleHandler>,
    on_hover_dropdown: Option<DropdownHoverHandler>,
    on_hover_option: Option<OptionHoverHandler>,
    on_close_dropdowns: Option<VoidHandler>,
    on_hover_tooltip: Option<TooltipHoverHandler>,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new(
            "system",
            false,
            false,
            false,
            DiscordRpcActivity::Disabled,
            true,
            UpdateState::Idle,
            None,
            false,
            None,
            None,
            None,
            None,
            None,
        )
    }
}

impl SettingsPage {
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        current_locale: &'static str,
        minimize_to_tray: bool,
        autostart: bool,
        autostart_to_tray: bool,
        discord_rpc: DiscordRpcActivity,
        check_updates: bool,
        update_state: UpdateState,
        open_dropdown: Option<&'static str>,
        open_dropdown_upward: bool,
        opening_dropdown: Option<&'static str>,
        closing_dropdown: Option<&'static str>,
        hovered_dropdown: Option<&'static str>,
        hovered_option: Option<(&'static str, &'static str)>,
        pending_selection: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            current_locale,
            minimize_to_tray,
            autostart,
            autostart_to_tray,
            discord_rpc,
            check_updates,
            update_state,
            open_dropdown,
            open_dropdown_upward,
            opening_dropdown,
            closing_dropdown,
            hovered_dropdown,
            hovered_option,
            pending_selection,
            on_change_palette: None,
            on_change_language: None,
            on_change_theme: None,
            on_change_transparency: None,
            on_toggle_minimize_to_tray: None,
            on_toggle_autostart: None,
            on_toggle_autostart_to_tray: None,
            on_change_discord_rpc: None,
            on_toggle_check_updates: None,
            on_check_update: None,
            on_download_and_install_update: None,
            on_toggle_dropdown: None,
            on_hover_dropdown: None,
            on_hover_option: None,
            on_close_dropdowns: None,
            on_hover_tooltip: None,
        }
    }

    #[must_use]
    pub fn on_change_palette(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_palette = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_language(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_language = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_theme(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_theme = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_transparency(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_transparency = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_minimize_to_tray(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_minimize_to_tray = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_autostart(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_autostart = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_autostart_to_tray(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_autostart_to_tray = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_change_discord_rpc(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_discord_rpc = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_check_updates(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_check_updates = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_check_update(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_check_update = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_download_and_install_update(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_download_and_install_update = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_toggle_dropdown(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_dropdown(
        mut self,
        handler: impl Fn(&'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_dropdown = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_option(
        mut self,
        handler: impl Fn(&'static str, &'static str, &bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_option = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_close_dropdowns(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close_dropdowns = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn on_hover_tooltip(
        mut self,
        handler: impl Fn(Option<TooltipState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hover_tooltip = Some(Arc::new(handler));
        self
    }
}
impl RenderOnce for SettingsPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get(cx);
        let route = AppRoute::Settings;

        let appearance_card = build_appearance_card(AppearanceCardParams {
            theme: &theme,
            current_locale: self.current_locale,
            open_dropdown: self.open_dropdown,
            opening_dropdown: self.opening_dropdown,
            closing_dropdown: self.closing_dropdown,
            open_dropdown_upward: self.open_dropdown_upward,
            hovered_dropdown: self.hovered_dropdown,
            hovered_option: self.hovered_option,
            pending_selection: self.pending_selection,
            on_change_palette: self.on_change_palette,
            on_change_language: self.on_change_language,
            on_change_theme: self.on_change_theme,
            on_change_transparency: self.on_change_transparency,
            on_toggle_dropdown: self.on_toggle_dropdown.clone(),
            on_hover_dropdown: self.on_hover_dropdown.clone(),
            on_hover_option: self.on_hover_option.clone(),
            on_close_dropdowns: self.on_close_dropdowns.clone(),
        });

        let behavior_card = build_behavior_card(BehaviorCardParams {
            theme: &theme,
            minimize_to_tray: self.minimize_to_tray,
            autostart: self.autostart,
            autostart_to_tray: self.autostart_to_tray,
            discord_rpc: self.discord_rpc,
            open_dropdown: self.open_dropdown,
            opening_dropdown: self.opening_dropdown,
            closing_dropdown: self.closing_dropdown,
            open_dropdown_upward: self.open_dropdown_upward,
            hovered_dropdown: self.hovered_dropdown,
            hovered_option: self.hovered_option,
            pending_selection: self.pending_selection,
            on_toggle_minimize_to_tray: self.on_toggle_minimize_to_tray,
            on_toggle_autostart: self.on_toggle_autostart,
            on_toggle_autostart_to_tray: self.on_toggle_autostart_to_tray,
            on_change_discord_rpc: self.on_change_discord_rpc,
            on_toggle_dropdown: self.on_toggle_dropdown,
            on_hover_dropdown: self.on_hover_dropdown,
            on_hover_option: self.on_hover_option,
            on_close_dropdowns: self.on_close_dropdowns,
        });

        let updates_card = build_updates_card(UpdatesCardParams {
            theme: &theme,
            update_state: &self.update_state,
            check_updates: self.check_updates,
            on_toggle_check_updates: self.on_toggle_check_updates,
            on_check_update: self.on_check_update,
            on_download_and_install_update: self.on_download_and_install_update,
            on_hover_tooltip: self.on_hover_tooltip,
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(PageHeader::new(route.title(), route.description()))
            .child(appearance_card)
            .child(behavior_card)
            .child(updates_card)
    }
}
