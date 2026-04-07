pub mod disable_mouse_acceleration;
pub mod fast_keyboard_repeat;

use crate::tweaks::Tweak;

use self::disable_mouse_acceleration::DisableMouseAccelerationTweak;
use self::fast_keyboard_repeat::FastKeyboardRepeatTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(DisableMouseAccelerationTweak::new()),
        Box::new(FastKeyboardRepeatTweak::new()),
    ]
}
