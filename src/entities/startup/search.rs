use strsim::levenshtein;

use crate::entities::startup::types::StartupEntry;

/// Checks if a startup entry matches the search query using direct substring and Levenshtein distance <= 3.
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

    let mut target_texts: Vec<String> = Vec::new();
    target_texts.push(entry.display_name.to_lowercase());
    target_texts.push(entry.name.to_lowercase());
    if let Some(ref publ) = entry.publisher {
        target_texts.push(publ.to_lowercase());
    }
    if let Some(ref cmd) = entry.command {
        target_texts.push(cmd.to_lowercase());
    }
    if let Some(ref target) = entry.target_path {
        target_texts.push(target.to_lowercase());
    }
    target_texts.push(entry.location_label.to_lowercase());

    // 1. Direct whole-query substring in any target field
    for target in &target_texts {
        if target.contains(&q) {
            return true;
        }
    }

    // 2. Multi-word query matching with Levenshtein distance <= 3
    q_words.iter().all(|qw| {
        // Direct substring of query word in any target text
        if target_texts.iter().any(|t| t.contains(qw)) {
            return true;
        }

        // Levenshtein distance <= 3 against target words
        for target in &target_texts {
            for tw in target.split_whitespace() {
                let clean_tw = tw.trim_matches(|c: char| !c.is_alphanumeric());
                if clean_tw.is_empty() {
                    continue;
                }

                // If query word is very short (1-2 chars), check prefix
                if qw.len() <= 2 {
                    if clean_tw.starts_with(qw) {
                        return true;
                    }
                } else if levenshtein(clean_tw, qw) <= 3 {
                    return true;
                }
            }
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::startup::{StartupScope, StartupSource, StartupStatus};

    fn make_test_entry(display_name: &str, publisher: Option<&str>) -> StartupEntry {
        StartupEntry {
            id: "test_1".to_string(),
            name: display_name.to_string(),
            display_name: display_name.to_string(),
            publisher: publisher.map(ToString::to_string),
            source: StartupSource::StartupFolder,
            scope: StartupScope::CurrentUser,
            status: StartupStatus::Enabled,
            command: None,
            target_path: None,
            icon_path: None,
            location_label: "Startup Folder".to_string(),
            raw_id: "test".to_string(),
        }
    }

    #[test]
    fn test_exact_match() {
        let entry = make_test_entry("Notion", Some("Notion Labs"));
        assert!(matches_startup_query(&entry, "notion"));
        assert!(matches_startup_query(&entry, "labs"));
    }

    #[test]
    fn test_levenshtein_distance_typos() {
        let entry = make_test_entry("Discord", Some("Discord Inc."));
        // Distance 1 typo ("discrod")
        assert!(matches_startup_query(&entry, "discrod"));
        // Distance 2 typo ("dizkord")
        assert!(matches_startup_query(&entry, "dizkord"));
        // Distance 3 typo ("dizkorb")
        assert!(matches_startup_query(&entry, "dizkorb"));
        // Distance > 3 ("completelydifferent")
        assert!(!matches_startup_query(&entry, "completelydifferent"));
    }

    #[test]
    fn test_empty_query() {
        let entry = make_test_entry("Spotify", Some("Spotify AB"));
        assert!(matches_startup_query(&entry, ""));
        assert!(matches_startup_query(&entry, "   "));
    }
}
