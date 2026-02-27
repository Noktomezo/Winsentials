use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const SYSTEM_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows\System";
const ENABLE_ACTIVITY_FEED: &str = "EnableActivityFeed";
const PUBLISH_USER_ACTIVITIES: &str = "PublishUserActivities";

pub struct DisableActivityFeedTweak {
  meta: TweakMeta,
}

impl DisableActivityFeedTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_activity_feed".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableActivityFeed.name".to_string(),
        description_key: "tweaks.disableActivityFeed.description".to_string(),
        details_key: "tweaks.disableActivityFeed.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableActivityFeedTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let activity = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      ENABLE_ACTIVITY_FEED,
    );
    let publish = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      PUBLISH_USER_ACTIVITIES,
    );
    let is_applied = activity.map(|v| v == 0).unwrap_or(false)
      && publish.map(|v| v == 0).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      ENABLE_ACTIVITY_FEED,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      PUBLISH_USER_ACTIVITIES,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      ENABLE_ACTIVITY_FEED,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SYSTEM_PATH,
      PUBLISH_USER_ACTIVITIES,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
