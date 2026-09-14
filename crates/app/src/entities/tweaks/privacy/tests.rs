use super::*;

#[test]
fn test_privacy_telemetry_queries_run_without_panic() {
    let _ = is_telemetry_and_diagnostics_disabled();
    let _ = is_app_compat_telemetry_disabled();
    let _ = is_ceip_sqm_disabled();
    let _ = is_windows_error_reporting_disabled();
}

#[test]
fn test_privacy_advertising_queries_run_without_panic() {
    let _ = is_advertising_id_disabled();
    let _ = is_activity_history_disabled();
    let _ = is_input_telemetry_disabled();
    let _ = is_cloud_speech_cortana_disabled();
}

#[test]
fn test_privacy_system_queries_run_without_panic() {
    let _ = is_search_bing_integration_disabled();
    let _ = is_cloud_sync_disabled();
    let _ = is_location_and_sensors_disabled();
    let _ = is_developer_telemetry_disabled();
    let _ = is_feedback_notifications_disabled();
    let _ = is_website_language_access_disabled();
    let _ = is_wifi_sense_disabled();
}

#[test]
fn test_privacy_app_permissions_queries_run_without_panic() {
    let _ = is_app_camera_mic_access_disabled();
    let _ = is_app_personal_data_access_disabled();
    let _ = is_app_file_system_access_disabled();
}

#[test]
fn test_service_disabled_nonexistent() {
    // A nonexistent service should return Ok(()) without error
    let res = service_helper::set_service_disabled("NonExistentTestService12345", true, 3);
    assert!(res.is_ok());
}
