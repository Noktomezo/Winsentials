use strsim::damerau_levenshtein;

/// Splits text into searchable word tokens, splitting by whitespace, punctuation, and camelCase boundaries.
/// Also preserves full unsplit alphanumeric chunks if camelCase splitting divided them.
#[must_use]
pub fn extract_word_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if c.is_alphanumeric() {
            // Check camelCase boundary (e.g. EdgeRemoval -> Edge, Removal; SnapKey -> Snap, Key)
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

    // Also include the full alphanumeric chunks (e.g. "SnapKey" -> "snapkey")
    for chunk in text.split(|c: char| !c.is_alphanumeric()) {
        let trimmed = chunk.trim().to_lowercase();
        if !trimmed.is_empty() && !tokens.contains(&trimmed) {
            tokens.push(trimmed);
        }
    }

    tokens
}

/// Computes the maximum allowed Levenshtein distance scaled by query word length (up to max 3).
#[must_use]
pub fn max_allowed_distance(query_len: usize) -> usize {
    match query_len {
        0..=2 => 0,
        3..=4 => 1,
        5..=7 => 2,
        _ => 3,
    }
}

/// Computes match score of a query word against a candidate token using exact, prefix, substring, or Damerau-Levenshtein (<= 3).
#[must_use]
pub fn score_token_match(query_word: &str, candidate_token: &str) -> Option<i32> {
    if query_word == candidate_token {
        return Some(1000);
    }
    if candidate_token.starts_with(query_word) {
        let len_penalty = i32::try_from(candidate_token.len()).unwrap_or(0);
        return Some(800 - len_penalty);
    }
    if candidate_token.contains(query_word) {
        let len_penalty = i32::try_from(candidate_token.len()).unwrap_or(0);
        return Some(700 - len_penalty);
    }

    let q_len = query_word.chars().count();
    let max_dist = max_allowed_distance(q_len);
    if max_dist > 0 {
        let tok_len = candidate_token.chars().count();
        if tok_len.abs_diff(q_len) <= max_dist {
            let dist = damerau_levenshtein(candidate_token, query_word);
            if dist <= max_dist {
                let dist_penalty = i32::try_from(dist).unwrap_or(0) * 100;
                return Some(500 - dist_penalty);
            }
        }
    }

    None
}
