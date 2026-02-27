use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const GPU_ENERGY_DRV_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\GpuEnergyDrv";
const START_VALUE: &str = "Start";

pub struct DisableGpuEnergyDrvTweak {
  meta: TweakMeta,
}

impl DisableGpuEnergyDrvTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_gpu_energy_drv".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.disableGpuEnergyDrv.name".to_string(),
        description_key: "tweaks.disableGpuEnergyDrv.description".to_string(),
        details_key: "tweaks.disableGpuEnergyDrv.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Medium,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableGpuEnergyDrvTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      GPU_ENERGY_DRV_PATH,
      START_VALUE,
    );
    let is_applied = value.map(|v| v == 4).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      GPU_ENERGY_DRV_PATH,
      START_VALUE,
      4,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      GPU_ENERGY_DRV_PATH,
      START_VALUE,
      2,
    )
    .map_err(|e| e.to_string())
  }
}
