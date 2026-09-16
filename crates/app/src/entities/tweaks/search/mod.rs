pub mod aliases;
#[cfg(test)]
mod tests;

use crate::entities::tweaks::{TweakCategory, get_all_tweaks};
use crate::features::navigation::AppRoute;
pub use aliases::{DROPDOWN_TWEAKS, get_tweak_keywords};

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

struct SearchCandidate {
    id: &'static str,
    title_key: &'static str,
    desc_key: &'static str,
    route: AppRoute,
    icon: &'static str,
}

fn score_candidate(
    query: &str,
    q_words: &[&str],
    candidate_id: &str,
    candidate_id_spaced: &str,
    title_active: &str,
    title_en: &str,
    title_ru: &str,
    desc_active: &str,
    desc_en: &str,
    desc_ru: &str,
    cat_name: &str,
    cat_en: &str,
    keywords: &[&str],
) -> i32 {
    // 1. Exact match on id, id_spaced, titles, or keywords
    if query == candidate_id || query == candidate_id_spaced {
        return 2000;
    }
    if query == title_active || query == title_en || query == title_ru {
        return 1800;
    }
    if keywords.contains(&query) {
        return 1700;
    }

    // 2. Starts with query
    if title_active.starts_with(query) || title_en.starts_with(query) || title_ru.starts_with(query)
    {
        return 1400 - i32::try_from(title_active.len()).unwrap_or(0);
    }
    if candidate_id.starts_with(query) || candidate_id_spaced.starts_with(query) {
        return 1300 - i32::try_from(candidate_id.len()).unwrap_or(0);
    }
    if keywords.iter().any(|kw| kw.starts_with(query)) {
        return 1200;
    }

    // 3. Substring contained in titles, id, or keywords
    if title_active.contains(query) || title_en.contains(query) || title_ru.contains(query) {
        return 1000 - i32::try_from(title_active.len()).unwrap_or(0);
    }
    if candidate_id_spaced.contains(query) || candidate_id.contains(query) {
        return 900;
    }
    if keywords.iter().any(|kw| kw.contains(query)) {
        return 850;
    }

    // 4. Multi-word match: each query word is found in titles, id, keywords, or descriptions
    if q_words.len() > 1 {
        let all_words_match = q_words.iter().all(|qw| {
            title_active.contains(qw)
                || title_en.contains(qw)
                || title_ru.contains(qw)
                || candidate_id_spaced.contains(qw)
                || keywords.iter().any(|kw| kw.contains(qw))
                || desc_active.contains(qw)
                || desc_en.contains(qw)
                || desc_ru.contains(qw)
        });
        if all_words_match {
            return 750;
        }
    }

    // 5. Category match
    if cat_name.contains(query) || cat_en.contains(query) {
        return 400;
    }

    // 6. Description match
    if desc_active.contains(query) || desc_en.contains(query) || desc_ru.contains(query) {
        return 250;
    }

    0
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
        if dt.min_build.is_none_or(|min| windows_build >= min) {
            candidates.push(SearchCandidate {
                id: dt.id,
                title_key: dt.title_key,
                desc_key: dt.desc_key,
                route: dt.route,
                icon: dt.icon,
            });
        }
    }

    let mut scored_results: Vec<(i32, TweakSearchResult)> = Vec::new();

    for candidate in candidates {
        let title = rust_i18n::t!(candidate.title_key).to_string();
        let description = rust_i18n::t!(candidate.desc_key).to_string();
        let category_name = candidate.route.title();
        let category_en = candidate.route.english_name();

        let title_en = rust_i18n::t!(candidate.title_key, locale = "en").to_string();
        let desc_en = rust_i18n::t!(candidate.desc_key, locale = "en").to_string();
        let title_ru = rust_i18n::t!(candidate.title_key, locale = "ru").to_string();
        let desc_ru = rust_i18n::t!(candidate.desc_key, locale = "ru").to_string();

        let id_spaced = candidate.id.replace('_', " ");
        let keywords = get_tweak_keywords(candidate.id);

        let score = score_candidate(
            &clean_query,
            &q_words,
            candidate.id,
            &id_spaced,
            &title.to_lowercase(),
            &title_en.to_lowercase(),
            &title_ru.to_lowercase(),
            &description.to_lowercase(),
            &desc_en.to_lowercase(),
            &desc_ru.to_lowercase(),
            &category_name.to_lowercase(),
            &category_en.to_lowercase(),
            keywords,
        );

        if score > 0 {
            scored_results.push((
                score,
                TweakSearchResult {
                    tweak_id: candidate.id,
                    title,
                    description,
                    category_name,
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
