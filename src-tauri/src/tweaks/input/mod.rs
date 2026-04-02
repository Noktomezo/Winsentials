pub mod fast_keyboard_repeat;

use crate::tweaks::Tweak;

use self::fast_keyboard_repeat::FastKeyboardRepeatTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![Box::new(FastKeyboardRepeatTweak::new())]
}
