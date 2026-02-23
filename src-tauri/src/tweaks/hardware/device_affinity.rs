use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use crate::wmi_queries::{
  Win32_NetworkAdapter, Win32_USBController, Win32_VideoController,
  get_wmi_connection,
};
use winreg::enums::*;

const AFFINITY_SUFFIX: &str =
  r"Device Parameters\Interrupt Management\Affinity Policy";
const DEVICE_POLICY: &str = "DevicePolicy";
const ASSIGNMENT_SET_OVERRIDE: &str = "AssignmentSetOverride";

pub struct DeviceAffinityTweak {
  meta: TweakMeta,
}

impl DeviceAffinityTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "device_affinity".to_string(),
        category: TweakCategory::Hardware,
        name_key: "tweaks.deviceAffinity.name".to_string(),
        description_key: "tweaks.deviceAffinity.description".to_string(),
        details_key: "tweaks.deviceAffinity.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::Medium,
        min_windows_build: None,
      },
    }
  }
}

fn check_device_affinity(
  pnp_id: &str,
  expected_assignment: &[u8],
) -> Option<bool> {
  let path = format!(
    r"SYSTEM\CurrentControlSet\Enum\{}\{}",
    pnp_id, AFFINITY_SUFFIX
  );

  let device_policy =
    registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY)?;
  let assignment = registry::read_reg_binary(
    HKEY_LOCAL_MACHINE,
    &path,
    ASSIGNMENT_SET_OVERRIDE,
  )?;

  let policy_ok = device_policy == 4;
  let assignment_ok = assignment == expected_assignment;

  Some(policy_ok && assignment_ok)
}

impl Tweak for DeviceAffinityTweak {
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
    let usbs: Vec<Win32_USBController> =
      wmi.query().map_err(|e| e.to_string())?;

    let mut any_device = false;
    let mut all_applied = true;

    for gpu in &gpus {
      if let Some(pnp_id) = &gpu.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          any_device = true;
          match check_device_affinity(pnp_id, &[0x02]) {
            Some(true) => {}
            Some(false) => all_applied = false,
            None => all_applied = false,
          }
        }
      }
    }

    for nic in &nics {
      if let Some(pnp_id) = &nic.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          any_device = true;
          match check_device_affinity(pnp_id, &[0x04]) {
            Some(true) => {}
            Some(false) => all_applied = false,
            None => all_applied = false,
          }
        }
      }
    }

    for usb in &usbs {
      if let Some(pnp_id) = &usb.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          any_device = true;
          match check_device_affinity(pnp_id, &[0x08]) {
            Some(true) => {}
            Some(false) => all_applied = false,
            None => all_applied = false,
          }
        }
      }
    }

    let is_applied = any_device && all_applied;
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
    let usbs: Vec<Win32_USBController> =
      wmi.query().map_err(|e| e.to_string())?;

    for gpu in &gpus {
      if let Some(pnp_id) = &gpu.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY, 4)
            .map_err(|e| e.to_string())?;
          registry::write_reg_binary(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
            &[0x02],
          )
          .ok();
        }
      }
    }

    for nic in &nics {
      if let Some(pnp_id) = &nic.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY, 4)
            .map_err(|e| e.to_string())?;
          registry::write_reg_binary(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
            &[0x04],
          )
          .ok();
        }
      }
    }

    for usb in &usbs {
      if let Some(pnp_id) = &usb.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY, 4)
            .map_err(|e| e.to_string())?;
          registry::write_reg_binary(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
            &[0x08],
          )
          .ok();
        }
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
    let usbs: Vec<Win32_USBController> =
      wmi.query().map_err(|e| e.to_string())?;

    for gpu in &gpus {
      if let Some(pnp_id) = &gpu.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY)
            .ok();
          registry::delete_reg_value(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
          )
          .ok();
        }
      }
    }

    for nic in &nics {
      if let Some(pnp_id) = &nic.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY)
            .ok();
          registry::delete_reg_value(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
          )
          .ok();
        }
      }
    }

    for usb in &usbs {
      if let Some(pnp_id) = &usb.PNPDeviceID {
        if pnp_id.starts_with("PCI\\VEN_") {
          let path = format!(
            r"SYSTEM\CurrentControlSet\Enum\{}\{}",
            pnp_id, AFFINITY_SUFFIX
          );
          registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, DEVICE_POLICY)
            .ok();
          registry::delete_reg_value(
            HKEY_LOCAL_MACHINE,
            &path,
            ASSIGNMENT_SET_OVERRIDE,
          )
          .ok();
        }
      }
    }

    Ok(())
  }
}
