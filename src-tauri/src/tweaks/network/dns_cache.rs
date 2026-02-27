use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const DNSCACHE_PARAMS_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters";
const NEGATIVE_CACHE_TIME: &str = "NegativeCacheTime";
const NEGATIVE_SOA_CACHE_TIME: &str = "NegativeSOACacheTime";
const NET_FAILURE_CACHE_TIME: &str = "NetFailureCacheTime";

pub struct DnsCacheTweak {
  meta: TweakMeta,
}

impl DnsCacheTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "dns_cache".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.dnsCache.name".to_string(),
        description_key: "tweaks.dnsCache.description".to_string(),
        details_key: "tweaks.dnsCache.details".to_string(),
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

impl Tweak for DnsCacheTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let neg_cache = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NEGATIVE_CACHE_TIME,
    );
    let is_applied = neg_cache.map(|v| v == 0).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NEGATIVE_CACHE_TIME,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NEGATIVE_SOA_CACHE_TIME,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NET_FAILURE_CACHE_TIME,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NEGATIVE_CACHE_TIME,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NEGATIVE_SOA_CACHE_TIME,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      DNSCACHE_PARAMS_PATH,
      NET_FAILURE_CACHE_TIME,
    )
    .ok();
    Ok(())
  }
}
