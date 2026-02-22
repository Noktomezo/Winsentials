mod csrss_priority;
mod mouse_acceleration_fix;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(mouse_acceleration_fix::MouseAccelerationFixTweak::new()),
    Box::new(csrss_priority::CsrssPriorityTweak::new()),
  ]
}
