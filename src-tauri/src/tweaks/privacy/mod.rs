mod app_privacy_tweaks;
mod disable_activity_feed;
mod disable_advertising;
mod disable_app_compat;
mod disable_ceip;
mod disable_cloud_search;
mod disable_compat_appraiser;
mod disable_consumer_features;
mod disable_cortana;
mod disable_handwriting;
mod disable_language_opt_out;
mod disable_location;
mod disable_online_speech;
mod disable_recall;
mod disable_search_location;
mod disable_sensors_wifi;
mod disable_setting_sync;
mod disable_silent_apps;
mod disable_suggestions;
mod disable_tailored_experiences;
mod disable_telemetry;
mod disable_telemetry_services;
mod disable_usage_tracking;
mod disable_voice_activation;
mod disable_wer;

use crate::tweaks::Tweak;
use app_privacy_tweaks::*;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(disable_telemetry::DisableTelemetryTweak::new()),
    Box::new(disable_telemetry_services::DisableTelemetryServicesTweak::new()),
    Box::new(disable_ceip::DisableCEIPTweak::new()),
    Box::new(disable_compat_appraiser::DisableCompatAppraiserTweak::new()),
    Box::new(disable_wer::DisableWERTweak::new()),
    Box::new(disable_app_compat::DisableAppCompatTweak::new()),
    Box::new(disable_advertising::DisableAdvertisingTweak::new()),
    Box::new(disable_silent_apps::DisableSilentAppsTweak::new()),
    Box::new(disable_suggestions::DisableSuggestionsTweak::new()),
    Box::new(
      disable_tailored_experiences::DisableTailoredExperiencesTweak::new(),
    ),
    Box::new(disable_recall::DisableRecallTweak::new()),
    Box::new(disable_cortana::DisableCortanaTweak::new()),
    Box::new(disable_location::DisableLocationTweak::new()),
    Box::new(disable_sensors_wifi::DisableSensorsWifiTweak::new()),
    Box::new(disable_search_location::DisableSearchLocationTweak::new()),
    Box::new(disable_voice_activation::DisableVoiceActivationTweak::new()),
    Box::new(disable_setting_sync::DisableSettingSyncTweak::new()),
    Box::new(disable_activity_feed::DisableActivityFeedTweak::new()),
    Box::new(disable_online_speech::DisableOnlineSpeechTweak::new()),
    Box::new(disable_cloud_search::DisableCloudSearchTweak::new()),
    Box::new(disable_consumer_features::DisableConsumerFeaturesTweak::new()),
    Box::new(disable_handwriting::DisableHandwritingTweak::new()),
    Box::new(disable_usage_tracking::DisableUsageTrackingTweak::new()),
    Box::new(disable_language_opt_out::DisableLanguageOptOutTweak::new()),
    Box::new(DenyCameraAccessTweak::new()),
    Box::new(DenyMicrophoneAccessTweak::new()),
    Box::new(DenyLocationAppAccessTweak::new()),
    Box::new(DenyContactsAccessTweak::new()),
    Box::new(DenyCalendarAccessTweak::new()),
    Box::new(DenyEmailAccessTweak::new()),
    Box::new(DenyMessagingAccessTweak::new()),
    Box::new(DenyPhoneAccessTweak::new()),
    Box::new(DenyCallHistoryAccessTweak::new()),
    Box::new(DenyTasksAccessTweak::new()),
    Box::new(DenyTrustedDevicesAccessTweak::new()),
    Box::new(DenyRadiosAccessTweak::new()),
    Box::new(DenyAccountInfoAccessTweak::new()),
    Box::new(DenyMotionAccessTweak::new()),
    Box::new(DenyUserDataAccessTweak::new()),
    Box::new(DenyVoiceActivationAccessTweak::new()),
    Box::new(DenyGraphicsCaptureAccessTweak::new()),
    Box::new(DenyNotificationsAccessTweak::new()),
  ]
}
