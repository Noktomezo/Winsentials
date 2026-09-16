use crate::entities::tweaks::{TweakCategory, get_all_tweaks};
use crate::features::navigation::AppRoute;

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

#[must_use]
pub fn search_tweaks(query: &str, windows_build: u32) -> Vec<TweakSearchResult> {
    let clean_query = query.trim().to_lowercase();
    if clean_query.is_empty() {
        return Vec::new();
    }

    let all_tweaks = get_all_tweaks();
    let mut scored_results: Vec<(i32, TweakSearchResult)> = Vec::new();

    for tweak in all_tweaks {
        if !tweak.is_supported(windows_build) {
            continue;
        }

        let title = rust_i18n::t!(tweak.title_key).to_string();
        let description = rust_i18n::t!(tweak.desc_key).to_string();
        let route = category_to_route(tweak.category);
        let category_name = route.title();
        let category_english = route.english_name();

        let title_lower = title.to_lowercase();
        let desc_lower = description.to_lowercase();
        let cat_lower = category_name.to_lowercase();
        let cat_en_lower = category_english.to_lowercase();
        let id_lower = tweak.id.to_lowercase();

        let mut score = 0;
        if title_lower.starts_with(&clean_query) {
            score = 1000 - i32::try_from(title.len()).unwrap_or(0);
        } else if title_lower.contains(&clean_query) {
            score = 800 - i32::try_from(title.len()).unwrap_or(0);
        } else if id_lower.contains(&clean_query) {
            score = 600;
        } else if cat_lower.contains(&clean_query) || cat_en_lower.contains(&clean_query) {
            score = 400;
        } else if desc_lower.contains(&clean_query) {
            score = 200;
        }

        if score > 0 {
            scored_results.push((
                score,
                TweakSearchResult {
                    tweak_id: tweak.id,
                    title,
                    description,
                    category_name,
                    route,
                    icon: tweak.icon,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_tweaks_empty_query() {
        let results = search_tweaks("", 22631);
        assert!(results.is_empty());

        let results = search_tweaks("   ", 22631);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_tweaks_finds_results() {
        let results = search_tweaks("context", 22631);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.route == AppRoute::ContextMenu));
        assert!(results.len() <= MAX_SEARCH_RESULTS);
    }

    #[test]
    fn test_search_tweaks_limited_to_five_items() {
        let results = search_tweaks("e", 22631);
        assert_eq!(results.len(), MAX_SEARCH_RESULTS);
    }

    #[test]
    fn test_search_tweaks_unsupported_build_filtered() {
        let results_old = search_tweaks("classic", 19041);
        let results_new = search_tweaks("classic", 22631);
        assert!(results_new.len() >= results_old.len());
    }

    #[test]
    fn test_category_to_route_coverage() {
        assert_eq!(
            category_to_route(TweakCategory::ContextMenu),
            AppRoute::ContextMenu
        );
        assert_eq!(
            category_to_route(TweakCategory::Explorer),
            AppRoute::Explorer
        );
        assert_eq!(
            category_to_route(TweakCategory::Interface),
            AppRoute::Interface
        );
        assert_eq!(category_to_route(TweakCategory::Input), AppRoute::Input);
        assert_eq!(category_to_route(TweakCategory::System), AppRoute::System);
        assert_eq!(
            category_to_route(TweakCategory::Network),
            AppRoute::NetworkTweaks
        );
        assert_eq!(category_to_route(TweakCategory::Privacy), AppRoute::Privacy);
    }
}
