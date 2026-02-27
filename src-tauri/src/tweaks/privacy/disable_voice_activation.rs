use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const WINDOWS_SEARCH_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\Windows Search";
const ALLOW_CORTANA: &str = "AllowCortana";
const SPEECH_PATH: &str = r"Software\Microsoft\Speech_OneCore\Preferences";
const VOICE_ACTIVATION_ON: &str = "VoiceActivationOn";

pub struct DisableVoiceActivationTweak {
  meta: TweakMeta,
}

impl DisableVoiceActivationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_voice_activation".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableVoiceActivation.name".to_string(),
        description_key: "tweaks.disableVoiceActivation.description"
          .to_string(),
        details_key: "tweaks.disableVoiceActivation.details".to_string(),
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

impl Tweak for DisableVoiceActivationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let cortana = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      ALLOW_CORTANA,
    );
    let voice = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      SPEECH_PATH,
      VOICE_ACTIVATION_ON,
    );
    let is_applied = cortana.map(|v| v == 0).unwrap_or(false)
      && voice.map(|v| v == 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      ALLOW_CORTANA,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      SPEECH_PATH,
      VOICE_ACTIVATION_ON,
      0,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      ALLOW_CORTANA,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      SPEECH_PATH,
      VOICE_ACTIVATION_ON,
      1,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }
}
