mod device_affinity;
mod msi_mode;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(msi_mode::MsiModeTweak::new()),
    Box::new(device_affinity::DeviceAffinityTweak::new()),
  ]
}
