use gpui::IntoElement;

use crate::entities::startup::StartupSource;
use crate::shared::theme::Theme;
use crate::shared::ui::Chip;
use super::types::FilterSelectHandler;
pub(crate) fn render_filter_pill(
    source: Option<StartupSource>,
    active_filter: Option<StartupSource>,
    _theme: &Theme,
    on_select_filter: Option<FilterSelectHandler>,
) -> impl IntoElement {
    let is_selected = active_filter == source;
    let label = match source {
        None => rust_i18n::t!("startup.filter_all").to_string(),
        Some(StartupSource::Registry) => rust_i18n::t!("startup.filter_registry").to_string(),
        Some(StartupSource::StartupFolder) => rust_i18n::t!("startup.filter_folder").to_string(),
        Some(StartupSource::Service) => rust_i18n::t!("startup.filter_services").to_string(),
        Some(StartupSource::ScheduledTask) => rust_i18n::t!("startup.filter_tasks").to_string(),
    };

    let pill_id = match source {
        None => "filter_all",
        Some(StartupSource::Registry) => "filter_reg",
        Some(StartupSource::StartupFolder) => "filter_folder",
        Some(StartupSource::Service) => "filter_svc",
        Some(StartupSource::ScheduledTask) => "filter_tasks",
    };

    Chip::new(pill_id, label)
        .selected(is_selected)
        .on_click(move |_event, window, cx| {
            if let Some(ref h) = on_select_filter {
                h(source, window, cx);
            }
        })
}

