use std::collections::HashMap;

use gpui::Global;

pub mod definitions;
pub mod types;

pub use definitions::ALL_TWEAKS;
pub use types::*;

#[must_use]
pub fn get_all_tweaks() -> &'static [TweakDefinition] {
    ALL_TWEAKS
}

#[must_use]
pub fn count_applied_tweaks(build: u32) -> (usize, usize) {
    let mut applied = 0;
    let mut total_supported = 0;

    for tweak in ALL_TWEAKS {
        if tweak.is_supported(build) {
            total_supported += 1;
            if (tweak.is_applied)() {
                applied += 1;
            }
        }
    }

    (applied, total_supported)
}

#[derive(Clone, Debug, Default)]
pub struct TweakStates {
    states: HashMap<&'static str, bool>,
}

impl Global for TweakStates {}

impl TweakStates {
    #[must_use]
    pub fn load_initial() -> Self {
        let mut states = HashMap::with_capacity(ALL_TWEAKS.len());
        for tweak in ALL_TWEAKS {
            states.insert(tweak.id, (tweak.is_applied)());
        }
        Self { states }
    }

    #[must_use]
    pub fn is_applied(&self, tweak: &TweakDefinition) -> bool {
        self.states
            .get(tweak.id)
            .copied()
            .unwrap_or_else(tweak.is_applied)
    }

    pub fn set_state(&mut self, tweak_id: &'static str, applied: bool) {
        self.states.insert(tweak_id, applied);
    }

    #[must_use]
    pub fn count_applied(&self, build: u32) -> (usize, usize) {
        let mut applied = 0;
        let mut total_supported = 0;

        for tweak in ALL_TWEAKS {
            if tweak.is_supported(build) {
                total_supported += 1;
                if self.is_applied(tweak) {
                    applied += 1;
                }
            }
        }

        (applied, total_supported)
    }
}

#[cfg(test)]
mod tests;