use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const MAX_BACKUP_NAME_LEN: usize = 40;

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

    #[must_use]
    pub fn calculate_diff(
        &self,
        current_states: &crate::entities::tweaks::TweakStates,
    ) -> BackupDiff {
        let mut to_enable = 0;
        let mut to_disable = 0;

        for tweak in crate::entities::tweaks::ALL_TWEAKS {
            if let Some(&desired) = self.tweak_states.get(tweak.id) {
                let current = current_states.is_applied(tweak);
                if desired && !current {
                    to_enable += 1;
                } else if !desired && current {
                    to_disable += 1;
                }
            }
        }

        BackupDiff {
            to_enable,
            to_disable,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackupDiff {
    pub to_enable: usize,
    pub to_disable: usize,
}

impl BackupDiff {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_enable == 0 && self.to_disable == 0
    }

    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.to_enable + self.to_disable
    }
}
