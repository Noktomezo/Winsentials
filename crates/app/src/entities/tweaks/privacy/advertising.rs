const REG_ADVERTISING_CU: &str = r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo";
const REG_ADVERTISING_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo";
const REG_CONTENT_DELIVERY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";

const REG_ACTIVITY_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\System";

const REG_INPUT_PERSONALIZATION_CU: &str = r"Software\Microsoft\Microsoft\InputPersonalization";
const REG_INPUT_PERSONALIZATION_OLD_CU: &str = r"Software\Microsoft\InputPersonalization";
const REG_INPUT_PERSONALIZATION_POLICIES: &str =
    r"SOFTWARE\Policies\Microsoft\InputPersonalization";
const REG_TRAINED_DATA_STORE: &str = r"Software\Microsoft\InputPersonalization\TrainedDataStore";
const REG_TIPC: &str = r"Software\Microsoft\Input\TIPC";

const REG_ONLINE_SPEECH: &str = r"Software\Microsoft\Speech_OneCore\Settings\OnlineSpeechPrivacy";
const REG_SPEECH_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Speech";
const REG_SEARCH_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\Windows Search";

// ----------------------------------------------------------------------------
// 5. Advertising ID and Recommendations
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_advertising_id_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_ADVERTISING_CU)
            .ok()
            .and_then(|k| k.get_u32("Enabled").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_advertising_id_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let cu_key = windows_registry::CURRENT_USER
            .create(REG_ADVERTISING_CU)
            .map_err(|e| format!("Failed to open AdvertisingInfo: {e}"))?;
        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_ADVERTISING_POLICIES)
            .map_err(|e| format!("Failed to open Advertising policy: {e}"))?;
        let cdm_key = windows_registry::CURRENT_USER
            .create(REG_CONTENT_DELIVERY)
            .map_err(|e| format!("Failed to open ContentDeliveryManager: {e}"))?;

        if applied {
            let _ = cu_key.set_u32("Enabled", 0);
            let _ = pol_key.set_u32("DisabledByGroupPolicy", 1);
            let _ = cdm_key.set_u32("SubscribedContent-338387Enabled", 0);
            let _ = cdm_key.set_u32("SubscribedContent-338388Enabled", 0);
            let _ = cdm_key.set_u32("SubscribedContent-338389Enabled", 0);
            let _ = cdm_key.set_u32("SystemPaneSuggestionsEnabled", 0);
        } else {
            let _ = cu_key.remove_value("Enabled");
            let _ = pol_key.remove_value("DisabledByGroupPolicy");
            let _ = cdm_key.remove_value("SubscribedContent-338387Enabled");
            let _ = cdm_key.remove_value("SubscribedContent-338388Enabled");
            let _ = cdm_key.remove_value("SubscribedContent-338389Enabled");
            let _ = cdm_key.remove_value("SystemPaneSuggestionsEnabled");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 6. Activity History & Timeline
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_activity_history_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_ACTIVITY_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("PublishUserActivities").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_activity_history_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_ACTIVITY_POLICIES)
            .map_err(|e| format!("Failed to open Activity policy: {e}"))?;

        if applied {
            let _ = key.set_u32("EnableActivityFeed", 0);
            let _ = key.set_u32("PublishUserActivities", 0);
            let _ = key.set_u32("UploadUserActivities", 0);
        } else {
            let _ = key.remove_value("EnableActivityFeed");
            let _ = key.remove_value("PublishUserActivities");
            let _ = key.remove_value("UploadUserActivities");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 7. Inking & Typing Personalization Telemetry
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_input_telemetry_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_INPUT_PERSONALIZATION_OLD_CU)
            .ok()
            .and_then(|k| k.get_u32("RestrictImplicitInkCollection").ok())
            == Some(1)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_input_telemetry_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let cu_key = windows_registry::CURRENT_USER
            .create(REG_INPUT_PERSONALIZATION_OLD_CU)
            .map_err(|e| format!("Failed to open InputPersonalization: {e}"))?;
        let cu_key2 = windows_registry::CURRENT_USER
            .create(REG_INPUT_PERSONALIZATION_CU)
            .map_err(|e| format!("Failed to open InputPersonalization v2: {e}"))?;
        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_INPUT_PERSONALIZATION_POLICIES)
            .map_err(|e| format!("Failed to open InputPersonalization policy: {e}"))?;
        let trained_key = windows_registry::CURRENT_USER
            .create(REG_TRAINED_DATA_STORE)
            .map_err(|e| format!("Failed to open TrainedDataStore: {e}"))?;
        let tipc_key = windows_registry::CURRENT_USER
            .create(REG_TIPC)
            .map_err(|e| format!("Failed to open TIPC: {e}"))?;

        if applied {
            let _ = cu_key.set_u32("RestrictImplicitInkCollection", 1);
            let _ = cu_key.set_u32("RestrictImplicitTextCollection", 1);
            let _ = cu_key2.set_u32("RestrictImplicitInkCollection", 1);
            let _ = cu_key2.set_u32("RestrictImplicitTextCollection", 1);
            let _ = pol_key.set_u32("AllowInputPersonalization", 0);
            let _ = pol_key.set_u32("RestrictImplicitInkCollection", 1);
            let _ = pol_key.set_u32("RestrictImplicitTextCollection", 1);
            let _ = trained_key.set_u32("HarvestContacts", 0);
            let _ = tipc_key.set_u32("Enabled", 0);
        } else {
            let _ = cu_key.remove_value("RestrictImplicitInkCollection");
            let _ = cu_key.remove_value("RestrictImplicitTextCollection");
            let _ = cu_key2.remove_value("RestrictImplicitInkCollection");
            let _ = cu_key2.remove_value("RestrictImplicitTextCollection");
            let _ = pol_key.remove_value("AllowInputPersonalization");
            let _ = pol_key.remove_value("RestrictImplicitInkCollection");
            let _ = pol_key.remove_value("RestrictImplicitTextCollection");
            let _ = trained_key.remove_value("HarvestContacts");
            let _ = tipc_key.remove_value("Enabled");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 8. Online Speech Recognition & Cortana Voice Activation
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_cloud_speech_cortana_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_ONLINE_SPEECH)
            .ok()
            .and_then(|k| k.get_u32("HasAccepted").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_cloud_speech_cortana_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let speech_cu = windows_registry::CURRENT_USER
            .create(REG_ONLINE_SPEECH)
            .map_err(|e| format!("Failed to open OnlineSpeechPrivacy: {e}"))?;
        let speech_pol = windows_registry::LOCAL_MACHINE
            .create(REG_SPEECH_POLICIES)
            .map_err(|e| format!("Failed to open Speech policy: {e}"))?;
        let search_pol = windows_registry::LOCAL_MACHINE
            .create(REG_SEARCH_POLICIES)
            .map_err(|e| format!("Failed to open Windows Search policy: {e}"))?;

        if applied {
            let _ = speech_cu.set_u32("HasAccepted", 0);
            let _ = speech_pol.set_u32("AllowSpeechModelUpdate", 0);
            let _ = search_pol.set_u32("AllowCortana", 0);
            let _ = search_pol.set_u32("AllowCortanaAboveLock", 0);
        } else {
            let _ = speech_cu.remove_value("HasAccepted");
            let _ = speech_pol.remove_value("AllowSpeechModelUpdate");
            let _ = search_pol.remove_value("AllowCortana");
            let _ = search_pol.remove_value("AllowCortanaAboveLock");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
