use strsim::levenshtein;

use crate::entities::startup::types::StartupEntry;

/// Splits text into searchable word tokens, splitting by whitespace, punctuation, and camelCase boundaries.
fn extract_word_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if c.is_alphanumeric() {
            // Check camelCase boundary (e.g. EdgeRemoval -> Edge, Removal; AMDInstall -> AMD, Install)
            if c.is_uppercase() && !current.is_empty() {
                let prev = chars[i - 1];
                let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
                if prev.is_lowercase() || (current.len() > 1 && next_is_lower) {
                    tokens.push(current.to_lowercase());
                    current.clear();
                }
            }
            current.push(c);
        } else if !current.is_empty() {
            tokens.push(current.to_lowercase());
            current.clear();
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }
    tokens
}

/// Computes the maximum allowed Levenshtein distance scaled by query word length (up to max 3).
fn max_allowed_distance(query_len: usize) -> usize {
    match query_len {
        0..=2 => 0,
        3..=4 => 1,
        5..=7 => 2,
        _ => 3,
    }
}

/// Checks if a startup entry matches the search query.
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
    let publisher = entry.publisher.as_deref().unwrap_or("").to_lowercase();
    let command = entry.command.as_deref().unwrap_or("").to_lowercase();
    let target_path = entry.target_path.as_deref().unwrap_or("").to_lowercase();
    let location_label = entry.location_label.to_lowercase();

    // 1. Direct whole-query substring match in any field
    if display_name.contains(&q)
        || name.contains(&q)
        || publisher.contains(&q)
        || command.contains(&q)
        || target_path.contains(&q)
        || location_label.contains(&q)
    {
        return true;
    }

    // Extract word tokens for fuzzy matching
    let mut candidate_tokens: Vec<String> = Vec::new();
    candidate_tokens.extend(extract_word_tokens(&entry.display_name));
    candidate_tokens.extend(extract_word_tokens(&entry.name));
    if let Some(ref publ) = entry.publisher {
        candidate_tokens.extend(extract_word_tokens(publ));
    }
    if let Some(ref target) = entry.target_path {
        if let Some(file_name) = std::path::Path::new(target)
            .file_name()
            .and_then(|n| n.to_str())
        {
            candidate_tokens.extend(extract_word_tokens(file_name));
        }
    }

    // 2. All query words must match either as substring or fuzzy match
    q_words.iter().all(|qw| {
        // Direct substring in any full field
        if display_name.contains(qw)
            || name.contains(qw)
            || publisher.contains(qw)
            || command.contains(qw)
            || target_path.contains(qw)
            || location_label.contains(qw)
        {
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
    fn test_exact_match() {
        let entry = make_test_entry("Notion", Some("Notion Labs"), None);
        assert!(matches_startup_query(&entry, "notion"));
        assert!(matches_startup_query(&entry, "labs"));
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
