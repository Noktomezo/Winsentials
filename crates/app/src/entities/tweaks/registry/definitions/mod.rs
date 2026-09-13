pub mod general;
pub mod privacy;

pub use general::GENERAL_TWEAKS;
pub use privacy::PRIVACY_TWEAKS;

use super::types::TweakDefinition;

const GENERAL_COUNT: usize = GENERAL_TWEAKS.len();
const PRIVACY_COUNT: usize = PRIVACY_TWEAKS.len();
const TOTAL_COUNT: usize = GENERAL_COUNT + PRIVACY_COUNT;

const COMBINED_TWEAKS: [TweakDefinition; TOTAL_COUNT] = {
    let mut arr = [GENERAL_TWEAKS[0]; TOTAL_COUNT];
    let mut i = 0;
    while i < GENERAL_COUNT {
        arr[i] = GENERAL_TWEAKS[i];
        i += 1;
    }
    let mut j = 0;
    while j < PRIVACY_COUNT {
        arr[i] = PRIVACY_TWEAKS[j];
        i += 1;
        j += 1;
    }
    arr
};

pub const ALL_TWEAKS: &[TweakDefinition] = &COMBINED_TWEAKS;
