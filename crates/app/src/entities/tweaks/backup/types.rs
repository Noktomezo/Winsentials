use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TweakBackup {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub timestamp_epoch_secs: i64,
    pub tweak_states: HashMap<String, bool>,
}

impl TweakBackup {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        created_at: impl Into<String>,
        timestamp_epoch_secs: i64,
        tweak_states: HashMap<String, bool>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            created_at: created_at.into(),
            timestamp_epoch_secs,
            tweak_states,
        }
    }

    #[must_use]
    pub fn active_count(&self) -> usize {
        self.tweak_states.values().filter(|&&v| v).count()
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        self.tweak_states.len()
    }
}
