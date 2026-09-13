pub mod advertising;
pub mod app_permissions;
pub mod service_helper;
pub mod system_privacy;
pub mod telemetry;

pub use advertising::{
    is_activity_history_disabled, is_advertising_id_disabled, is_cloud_speech_cortana_disabled,
    is_input_telemetry_disabled, set_activity_history_disabled, set_advertising_id_disabled,
    set_cloud_speech_cortana_disabled, set_input_telemetry_disabled,
};
pub use app_permissions::{
    is_app_camera_mic_access_disabled, is_app_file_system_access_disabled,
    is_app_personal_data_access_disabled, set_app_camera_mic_access_disabled,
    set_app_file_system_access_disabled, set_app_personal_data_access_disabled,
};
pub use system_privacy::{
    is_cloud_sync_disabled, is_developer_telemetry_disabled, is_feedback_notifications_disabled,
    is_location_and_sensors_disabled, is_search_bing_integration_disabled,
    is_website_language_access_disabled, is_wifi_sense_disabled, set_cloud_sync_disabled,
    set_developer_telemetry_disabled, set_feedback_notifications_disabled,
    set_location_and_sensors_disabled, set_search_bing_integration_disabled,
    set_website_language_access_disabled, set_wifi_sense_disabled,
};
pub use telemetry::{
    is_app_compat_telemetry_disabled, is_ceip_sqm_disabled, is_telemetry_and_diagnostics_disabled,
    is_windows_error_reporting_disabled, set_app_compat_telemetry_disabled, set_ceip_sqm_disabled,
    set_telemetry_and_diagnostics_disabled, set_windows_error_reporting_disabled,
};

#[cfg(test)]
mod tests;
