use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const APP_PRIVACY_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\AppPrivacy";

macro_rules! app_privacy_tweak {
  ($id:expr, $name_key:expr, $desc_key:expr, $reg_value:expr, $min_build:expr) => {
    paste::paste! {
      pub struct [<Deny $id AccessTweak>] {
        meta: TweakMeta,
      }

      impl [<Deny $id AccessTweak>] {
        pub fn new() -> Self {
          Self {
            meta: TweakMeta {
              id: stringify!([<deny_ $id:snake _access>]).to_string(),
              category: TweakCategory::Privacy,
              name_key: format!("tweaks.deny{}Access.name", $name_key),
              description_key: format!("tweaks.deny{}Access.description", $desc_key),
              details_key: format!("tweaks.deny{}Access.details", $desc_key),
              ui_type: TweakUiType::Toggle,
              options: vec![],
              requires_reboot: false,
              requires_logout: false,
              risk_level: RiskLevel::Low,
              min_windows_build: $min_build,
            },
          }
        }
      }

      impl Tweak for [<Deny $id AccessTweak>] {
        fn meta(&self) -> &TweakMeta {
          &self.meta
        }

        fn check(&self) -> Result<TweakState, String> {
          let value = registry::read_reg_u32(HKEY_LOCAL_MACHINE, APP_PRIVACY_PATH, $reg_value);
          let is_applied = value.map(|v| v == 2).unwrap_or(false);
          Ok(TweakState {
            id: self.meta.id.clone(),
            current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
            is_applied,
          })
        }

        fn apply(&self, _value: Option<&str>) -> Result<(), String> {
          registry::write_reg_u32(HKEY_LOCAL_MACHINE, APP_PRIVACY_PATH, $reg_value, 2)
            .map_err(|e| e.to_string())
        }

        fn revert(&self) -> Result<(), String> {
          registry::write_reg_u32(HKEY_LOCAL_MACHINE, APP_PRIVACY_PATH, $reg_value, 0)
            .map_err(|e| e.to_string())
        }
      }
    }
  };
}

app_privacy_tweak!(Camera, "Camera", "Camera", "LetAppsAccessCamera", None);
app_privacy_tweak!(
  Microphone,
  "Microphone",
  "Microphone",
  "LetAppsAccessMicrophone",
  None
);
app_privacy_tweak!(
  LocationApp,
  "LocationApp",
  "LocationApp",
  "LetAppsAccessLocation",
  None
);
app_privacy_tweak!(
  Contacts,
  "Contacts",
  "Contacts",
  "LetAppsAccessContacts",
  None
);
app_privacy_tweak!(
  Calendar,
  "Calendar",
  "Calendar",
  "LetAppsAccessCalendar",
  None
);
app_privacy_tweak!(Email, "Email", "Email", "LetAppsAccessEmail", None);
app_privacy_tweak!(
  Messaging,
  "Messaging",
  "Messaging",
  "LetAppsAccessMessaging",
  None
);
app_privacy_tweak!(Phone, "Phone", "Phone", "LetAppsAccessPhone", None);
app_privacy_tweak!(
  CallHistory,
  "CallHistory",
  "CallHistory",
  "LetAppsAccessCallHistory",
  None
);
app_privacy_tweak!(Tasks, "Tasks", "Tasks", "LetAppsAccessTasks", None);
app_privacy_tweak!(
  TrustedDevices,
  "TrustedDevices",
  "TrustedDevices",
  "LetAppsAccessTrustedDevices",
  None
);
app_privacy_tweak!(Radios, "Radios", "Radios", "LetAppsAccessRadios", None);
app_privacy_tweak!(
  AccountInfo,
  "AccountInfo",
  "AccountInfo",
  "LetAppsAccessAccountInfo",
  None
);
app_privacy_tweak!(Motion, "Motion", "Motion", "LetAppsAccessMotion", None);
app_privacy_tweak!(
  UserData,
  "UserData",
  "UserData",
  "LetAppsAccessUserData",
  None
);
app_privacy_tweak!(
  VoiceActivation,
  "VoiceActivation",
  "VoiceActivation",
  "LetAppsAccessVoiceActivation",
  None
);
app_privacy_tweak!(
  GraphicsCapture,
  "GraphicsCapture",
  "GraphicsCapture",
  "LetAppsAccessGraphicsCaptureProgrammatic",
  None
);

const WIN11_BUILD: u32 = 22000;
app_privacy_tweak!(
  Notifications,
  "Notifications",
  "Notifications",
  "LetAppsAccessNotifications",
  Some(WIN11_BUILD)
);
