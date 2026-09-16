#[cfg(test)]
mod tests;

use crate::entities::tweaks::{TweakCategory, get_all_tweaks};
use crate::features::navigation::AppRoute;
use crate::shared::fuzzy::{extract_word_tokens, score_token_match};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TweakSearchResult {
    pub tweak_id: &'static str,
    pub title: String,
    pub description: String,
    pub category_name: String,
    pub route: AppRoute,
    pub icon: &'static str,
}

#[must_use]
pub const fn category_to_route(category: TweakCategory) -> AppRoute {
    match category {
        TweakCategory::ContextMenu => AppRoute::ContextMenu,
        TweakCategory::Explorer => AppRoute::Explorer,
        TweakCategory::Interface => AppRoute::Interface,
        TweakCategory::Input => AppRoute::Input,
        TweakCategory::System | TweakCategory::Performance => AppRoute::System,
        TweakCategory::Network => AppRoute::NetworkTweaks,
        TweakCategory::Privacy => AppRoute::Privacy,
    }
}

pub const MAX_SEARCH_RESULTS: usize = 5;

pub struct DropdownTweakMeta {
    pub id: &'static str,
    pub route: AppRoute,
    pub icon: &'static str,
    pub title_key: &'static str,
    pub desc_key: &'static str,
}

pub const DROPDOWN_TWEAKS: [DropdownTweakMeta; 3] = [
    DropdownTweakMeta {
        id: "keyboard_repeat",
        route: AppRoute::Input,
        icon: "icons/keyboard.svg",
        title_key: "tweaks.keyboard_repeat_title",
        desc_key: "tweaks.keyboard_repeat_desc",
    },
    DropdownTweakMeta {
        id: "ctf_optimization",
        route: AppRoute::Input,
        icon: "icons/type.svg",
        title_key: "tweaks.ctf_optimization_title",
        desc_key: "tweaks.ctf_optimization_desc",
    },
    DropdownTweakMeta {
        id: "snapkey",
        route: AppRoute::Input,
        icon: "icons/crosshair-2.svg",
        title_key: "tweaks.snapkey_title",
        desc_key: "tweaks.snapkey_desc",
    },
];

struct SearchCandidate {
    id: &'static str,
    title_key: &'static str,
    desc_key: &'static str,
    route: AppRoute,
    icon: &'static str,
}

#[must_use]
pub fn search_tweaks(query: &str, windows_build: u32) -> Vec<TweakSearchResult> {
    let clean_query = query.trim().to_lowercase();
    if clean_query.is_empty() {
        return Vec::new();
    }
    let q_words: Vec<&str> = clean_query.split_whitespace().collect();

    let mut candidates: Vec<SearchCandidate> = Vec::new();

    // Standard binary tweaks from registry
    for tweak in get_all_tweaks() {
        if tweak.is_supported(windows_build) {
            candidates.push(SearchCandidate {
                id: tweak.id,
                title_key: tweak.title_key,
                desc_key: tweak.desc_key,
                route: category_to_route(tweak.category),
                icon: tweak.icon,
            });
        }
    }

    // Dropdown input tweaks (SnapKey, CTF, Keyboard Repeat)
    for dt in &DROPDOWN_TWEAKS {
        candidates.push(SearchCandidate {
            id: dt.id,
            title_key: dt.title_key,
            desc_key: dt.desc_key,
            route: dt.route,
            icon: dt.icon,
        });
    }

    let mut scored_results: Vec<(i32, TweakSearchResult)> = Vec::new();

    for candidate in candidates {
        let mut candidate_tokens: Vec<String> = Vec::new();
        let mut localized_titles: Vec<String> = Vec::new();
        let mut localized_descs: Vec<String> = Vec::new();

        // 1. Add ID tokens (e.g. "disable_mouse_acceleration" -> ["disable", "mouse", "acceleration", "disable mouse acceleration"])
        candidate_tokens.extend(extract_word_tokens(candidate.id));
        let id_spaced = candidate.id.replace('_', " ");
        if !candidate_tokens.contains(&id_spaced) {
            candidate_tokens.push(id_spaced.clone());
        }

        // 2. Add tokens from titles across ALL available locales dynamically
        for loc in rust_i18n::available_locales!() {
            let title = rust_i18n::t!(candidate.title_key, locale = loc).to_string();
            candidate_tokens.extend(extract_word_tokens(&title));
            localized_titles.push(title.to_lowercase());

            let desc = rust_i18n::t!(candidate.desc_key, locale = loc).to_string();
            localized_descs.push(desc.to_lowercase());
        }

        let mut score = 0;

        // Check exact match on candidate ID or ID with spaces
        if clean_query == candidate.id || clean_query == id_spaced {
            score = 3000;
        }

        // Check whole query against titles across all locales
        for title_lower in &localized_titles {
            if *title_lower == clean_query {
                score = score.max(2800);
            } else if title_lower.starts_with(&clean_query) {
                let len_penalty = i32::try_from(title_lower.len()).unwrap_or(0);
                score = score.max(2500 - len_penalty);
            } else if title_lower.contains(&clean_query) {
                let len_penalty = i32::try_from(title_lower.len()).unwrap_or(0);
                score = score.max(2000 - len_penalty);
            }
        }

        // Multi-word / token matching with Levenshtein distance up to 3
        if score == 0 {
            let mut all_words_matched = true;
            let mut word_scores_sum = 0;

            for qw in &q_words {
                let mut best_word_score = None;
                for tok in &candidate_tokens {
                    if let Some(s) = score_token_match(qw, tok) {
                        best_word_score = Some(best_word_score.map_or(s, |b: i32| b.max(s)));
                    }
                }

                if let Some(ws) = best_word_score {
                    word_scores_sum += ws;
                } else {
                    all_words_matched = false;
                    break;
                }
            }

            if all_words_matched && !q_words.is_empty() {
                score = word_scores_sum;
            }
        }

        // Fallback: search descriptions across all locales
        if score == 0 {
            for desc_lower in &localized_descs {
                if desc_lower.contains(&clean_query) {
                    score = 150;
                    break;
                }
            }
        }

        if score > 0 {
            scored_results.push((
                score,
                TweakSearchResult {
                    tweak_id: candidate.id,
                    title: rust_i18n::t!(candidate.title_key).to_string(),
                    description: rust_i18n::t!(candidate.desc_key).to_string(),
                    category_name: candidate.route.title(),
                    route: candidate.route,
                    icon: candidate.icon,
                },
            ));
        }
    }

    scored_results.sort_by_key(|a| std::cmp::Reverse(a.0));
    scored_results
        .into_iter()
        .take(MAX_SEARCH_RESULTS)
        .map(|(_, item)| item)
        .collect()
}
