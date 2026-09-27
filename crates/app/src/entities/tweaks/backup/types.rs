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
    #[serde(default)]
    pub dropdown_states: HashMap<String, String>,
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
            dropdown_states: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_dropdown_states(mut self, dropdown_states: HashMap<String, String>) -> Self {
        self.dropdown_states = dropdown_states;
        self
    }

    #[must_use]
    pub fn active_count(&self) -> usize {
        let binary_active = self.tweak_states.values().filter(|&&v| v).count();
        let dropdown_active = self
            .dropdown_states
            .values()
            .filter(|val| *val != "off" && *val != "standard")
            .count();
        binary_active + dropdown_active
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        self.tweak_states.len() + self.dropdown_states.len()
    }

    #[must_use]
    pub fn calculate_diff_full(
        &self,
        current_states: &crate::entities::tweaks::TweakStates,
        current_dropdowns: &HashMap<String, String>,
    ) -> BackupDiff {
        let mut to_enable = 0;
        let mut to_disable = 0;
        let mut to_change_presets = 0;

        for tweak in crate::entities::tweaks::ALL_TWEAKS {
            match tweak.kind {
                crate::entities::tweaks::TweakKind::Toggle { .. } => {
                    if let Some(&desired) = self.tweak_states.get(tweak.id) {
                        let current = current_states.is_applied(tweak);
                        if desired && !current {
                            to_enable += 1;
                        } else if !desired && current {
                            to_disable += 1;
                        }
                    }
                }
                crate::entities::tweaks::TweakKind::Select { .. } => {
                    if let Some(desired_preset) = self.dropdown_states.get(tweak.id) {
                        if let Some(current_preset) = current_dropdowns.get(tweak.id) {
                            if current_preset != desired_preset {
                                to_change_presets += 1;
                            }
                        }
                    }
                }
            }
        }

        BackupDiff {
            to_enable,
            to_disable,
            to_change_presets,
        }
    }

    #[must_use]
    pub fn calculate_diff(
        &self,
        current_states: &crate::entities::tweaks::TweakStates,
    ) -> BackupDiff {
        let mut current_dropdowns = HashMap::new();
        for tweak in crate::entities::tweaks::ALL_TWEAKS {
            if let crate::entities::tweaks::TweakKind::Select { get_current, .. } = tweak.kind {
                current_dropdowns.insert(tweak.id.to_string(), get_current().to_string());
            }
        }
        self.calculate_diff_full(current_states, &current_dropdowns)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackupDiff {
    pub to_enable: usize,
    pub to_disable: usize,
    pub to_change_presets: usize,
}

impl BackupDiff {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_enable == 0 && self.to_disable == 0 && self.to_change_presets == 0
    }

    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.to_enable + self.to_disable + self.to_change_presets
    }
}
