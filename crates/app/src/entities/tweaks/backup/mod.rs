pub mod storage;
pub mod time;
pub mod types;

pub use storage::*;
pub use time::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::entities::tweaks::ALL_TWEAKS;

    #[test]
    fn test_backup_types_creation_and_counts() {
        let mut states = HashMap::new();
        states.insert("tweak1".to_string(), true);
        states.insert("tweak2".to_string(), false);
        states.insert("tweak3".to_string(), true);

        let backup = TweakBackup::new(
            "id-123",
            "Test Backup",
            "11.09.2026, 00:00:00",
            1234567,
            states,
        );
        assert_eq!(backup.id, "id-123");
        assert_eq!(backup.name, "Test Backup");
        assert_eq!(backup.active_count(), 2);
        assert_eq!(backup.total_count(), 3);
    }

    #[test]
    fn test_forward_compatibility_migration() {
        let mut states = HashMap::new();
        states.insert("some_old_tweak".to_string(), true);

        let mut backup =
            TweakBackup::new("mig-1", "Old Backup", "10.09.2026, 12:00:00", 1000, states);

        // Migrate using ALL_TWEAKS
        for tweak in ALL_TWEAKS {
            if !backup.tweak_states.contains_key(tweak.id) {
                backup.tweak_states.insert(tweak.id.to_string(), false);
            }
        }

        for tweak in ALL_TWEAKS {
            assert!(backup.tweak_states.contains_key(tweak.id));
            assert_eq!(backup.tweak_states[tweak.id], false);
        }
        assert_eq!(backup.tweak_states["some_old_tweak"], true);
    }

    #[test]
    fn test_local_time_formatting() {
        let (formatted, epoch) = format_local_time_now();
        assert!(!formatted.is_empty());
        assert!(epoch > 0);
    }

    #[test]
    fn test_backup_serialization() {
        let mut states = HashMap::new();
        states.insert("tweak_a".to_string(), true);
        let backup = TweakBackup::new(
            "ser-1",
            "Ser Backup",
            "11.09.2026, 00:15:30",
            1700000000,
            states,
        );

        let json = serde_json::to_string(&backup).expect("serialize");
        let deserialized: TweakBackup = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(backup, deserialized);
    }
}
