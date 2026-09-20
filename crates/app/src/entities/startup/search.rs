use strsim::levenshtein;

use crate::entities::startup::types::StartupEntry;
use crate::shared::fuzzy::{extract_word_tokens, max_allowed_distance};

/// Checks if a startup entry matches the search query strictly by its name / display name.
#[must_use]
pub fn matches_startup_query(entry: &StartupEntry, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }

    let q_words: Vec<&str> = q.split_whitespace().collect();
    if q_words.is_empty() {
        return true;
    }

    let display_name = entry.display_name.to_lowercase();
    let name = entry.name.to_lowercase();

    // 1. Direct whole-query substring match in names
    if display_name.contains(&q) || name.contains(&q) {
        return true;
    }

    // Extract word tokens from names for fuzzy matching
    let mut candidate_tokens: Vec<String> = Vec::new();
    candidate_tokens.extend(extract_word_tokens(&entry.display_name));
    candidate_tokens.extend(extract_word_tokens(&entry.name));

    // 2. All query words must match either as substring or fuzzy match against names
    q_words.iter().all(|qw| {
        // Direct substring in names
        if display_name.contains(qw) || name.contains(qw) {
            return true;
        }

        let max_dist = max_allowed_distance(qw.chars().count());
        if max_dist == 0 {
            return candidate_tokens.iter().any(|tok| tok.starts_with(qw));
        }

        // Fuzzy Levenshtein match on candidate tokens
        candidate_tokens.iter().any(|tok| {
            if tok.starts_with(qw) || qw.starts_with(tok) {
                return true;
            }
            let tok_len = tok.chars().count();
            let qw_len = qw.chars().count();
            if tok_len.abs_diff(qw_len) <= max_dist {
                levenshtein(tok, qw) <= max_dist
            } else {
                false
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::startup::{StartupScope, StartupSource, StartupStatus};

    fn make_test_entry(
        display_name: &str,
        publisher: Option<&str>,
        command: Option<&str>,
    ) -> StartupEntry {
        StartupEntry {
            id: "test_1".to_string(),
            name: display_name.to_string(),
            display_name: display_name.to_string(),
            publisher: publisher.map(ToString::to_string),
            source: StartupSource::ScheduledTask,
            scope: StartupScope::CurrentUser,
            status: StartupStatus::Enabled,
            command: command.map(ToString::to_string),
            target_path: command.map(ToString::to_string),
            icon_path: None,
            location_label: "Task Scheduler".to_string(),
            raw_id: "test".to_string(),
        }
    }

    #[test]
    fn test_name_only_match() {
        let entry = make_test_entry(
            "Discord",
            Some("Discord Inc."),
            Some("C:\\Discord\\Update.exe"),
        );
        assert!(matches_startup_query(&entry, "discord"));
        assert!(matches_startup_query(&entry, "discrod")); // Levenshtein typo
        // Searching publisher or path should NOT match
        assert!(!matches_startup_query(&entry, "Update"));
        assert!(!matches_startup_query(&entry, "Inc"));
    }

    #[test]
    fn test_desk_query_does_not_match_unrelated_items() {
        let edge = make_test_entry(
            "EdgeRemoval",
            Some("Microsoft"),
            Some("C:\\Windows\\System32\\cmd.exe /c del edge"),
        );
        assert!(!matches_startup_query(&edge, "Desk"));

        let amd = make_test_entry(
            "AMD Install Manager",
            Some("Advanced Micro Devices, Inc."),
            Some("C:\\Program Files\\AMD\\CNext\\CNext\\setup.exe"),
        );
        assert!(!matches_startup_query(&amd, "Desk"));
    }

    #[test]
    fn test_levenshtein_distance_typos() {
        let entry = make_test_entry("Discord", Some("Discord Inc."), None);
        // Distance 1 typo ("discrod")
        assert!(matches_startup_query(&entry, "discrod"));
        // Distance 2 typo ("dizkord")
        assert!(matches_startup_query(&entry, "dizkord"));
        // Distance > 2 for 7-char word ("dizkorb" distance 3)
        assert!(!matches_startup_query(&entry, "dizkorb"));
        // Distance > 3 ("completelydifferent")
        assert!(!matches_startup_query(&entry, "completelydifferent"));
    }

    #[test]
    fn test_empty_query() {
        let entry = make_test_entry("Spotify", Some("Spotify AB"), None);
        assert!(matches_startup_query(&entry, ""));
        assert!(matches_startup_query(&entry, "   "));
    }
}
