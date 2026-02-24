use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use crate::wmi_queries::{
  get_wmi_connection, Win32_NetworkAdapter, Win32_VideoController,
};
use winreg::enums::*;

const MSI_PROPS_SUFFIX: &str =
  r"Device Parameters\Interrupt Management\MessageSignaledInterruptProperties";
const AFFINITY_SUFFIX: &str =
  r"Device Parameters\Interrupt Management\Affinity Policy";
const MSI_SUPPORTED: &str = "MSISupported";
const DEVICE_PRIORITY: &str = "DevicePriority";

pub struct MsiModeTweak {
  meta: TweakMeta,
}

impl MsiModeTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "msi_mode".to_string(),
        category: TweakCategory::Hardware,
        name_key: "tweaks.msiMode.name".to_string(),
        description_key: "tweaks.msiMode.description".to_string(),
        details_key: "tweaks.msiMode.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::Medium,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for MsiModeTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let Some(wmi) = get_wmi_connection() else {
      return Err("Cannot connect to WMI".to_string());
    };

    let gpus: Vec<Win32_VideoController> =
      wmi.query().map_err(|e| e.to_string())?;
    let nics: Vec<Win32_NetworkAdapter> =
      wmi.query().map_err(|e| e.to_string())?;

    let mut all_enabled = true;
    let mut any_device = false;

    for gpu in gpus {
      if let Some(pnp_id) = gpu.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        any_device = true;
        let path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        let msi_supported =
          registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, MSI_SUPPORTED);
        if msi_supported != Some(1) {
          all_enabled = false;
        }
      }
    }

    for nic in nics {
      if let Some(pnp_id) = nic.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        any_device = true;
        let path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        let msi_supported =
          registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, MSI_SUPPORTED);
        if msi_supported != Some(1) {
          all_enabled = false;
        }
      }
    }

    let is_applied = any_device && all_enabled;
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    let Some(wmi) = get_wmi_connection() else {
      return Err("Cannot connect to WMI".to_string());
    };

    let gpus: Vec<Win32_VideoController> =
      wmi.query().map_err(|e| e.to_string())?;
    let nics: Vec<Win32_NetworkAdapter> =
      wmi.query().map_err(|e| e.to_string())?;

    for gpu in gpus {
      if let Some(pnp_id) = gpu.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        let msi_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        registry::write_reg_u32(
          HKEY_LOCAL_MACHINE,
          &msi_path,
          MSI_SUPPORTED,
          1,
        )
        .map_err(|e| e.to_string())?;

        let affinity_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, AFFINITY_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &affinity_path,
          DEVICE_PRIORITY,
        )
        .ok();
      }
    }

    for nic in nics {
      if let Some(pnp_id) = nic.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        let msi_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        registry::write_reg_u32(
          HKEY_LOCAL_MACHINE,
          &msi_path,
          MSI_SUPPORTED,
          1,
        )
        .map_err(|e| e.to_string())?;

        let affinity_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, AFFINITY_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &affinity_path,
          DEVICE_PRIORITY,
        )
        .ok();
      }
    }

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    let Some(wmi) = get_wmi_connection() else {
      return Err("Cannot connect to WMI".to_string());
    };

    let gpus: Vec<Win32_VideoController> =
      wmi.query().map_err(|e| e.to_string())?;
    let nics: Vec<Win32_NetworkAdapter> =
      wmi.query().map_err(|e| e.to_string())?;

    for gpu in gpus {
      if let Some(pnp_id) = gpu.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        let msi_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &msi_path,
          MSI_SUPPORTED,
        )
        .ok();

        let affinity_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, AFFINITY_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &affinity_path,
          DEVICE_PRIORITY,
        )
        .ok();
      }
    }

    for nic in nics {
      if let Some(pnp_id) = nic.PNPDeviceID
        && pnp_id.starts_with("PCI\\VEN_")
      {
        let msi_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, MSI_PROPS_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &msi_path,
          MSI_SUPPORTED,
        )
        .ok();

        let affinity_path = format!(
          r"SYSTEM\CurrentControlSet\Enum\{}\{}",
          pnp_id, AFFINITY_SUFFIX
        );
        registry::delete_reg_value(
          HKEY_LOCAL_MACHINE,
          &affinity_path,
          DEVICE_PRIORITY,
        )
        .ok();
      }
    }

    Ok(())
  }
}
