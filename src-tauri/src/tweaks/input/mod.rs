pub mod csrss_high_priority;
pub mod disable_mouse_acceleration;
pub mod fast_keyboard_repeat;
pub mod raw_mouse_throttle;

use crate::tweaks::Tweak;

use self::csrss_high_priority::CsrssHighPriorityTweak;
use self::disable_mouse_acceleration::DisableMouseAccelerationTweak;
use self::fast_keyboard_repeat::FastKeyboardRepeatTweak;
use self::raw_mouse_throttle::RawMouseThrottleTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(CsrssHighPriorityTweak::new()),
        Box::new(DisableMouseAccelerationTweak::new()),
        Box::new(FastKeyboardRepeatTweak::new()),
        Box::new(RawMouseThrottleTweak::new()),
    ]
}
