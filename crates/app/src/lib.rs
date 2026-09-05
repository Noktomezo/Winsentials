#![allow(
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::too_many_arguments,
    clippy::ref_option,
    clippy::needless_pass_by_value
)]
pub mod app;
pub mod entities;
pub mod features;
pub mod pages;
pub mod shared;
pub mod widgets;

rust_i18n::i18n!("../../locales", fallback = "en");

#[cfg(test)]
mod tests {
    #[test]
    fn test_network_tweaks_i18n_keys() {
        rust_i18n::set_locale("ru");
        assert_ne!(rust_i18n::t!("tweaks.bbr2_title"), "tweaks.bbr2_title");
        assert_ne!(rust_i18n::t!("tweaks.rss_title"), "tweaks.rss_title");
        assert_ne!(
            rust_i18n::t!("tweaks.fast_send_copy_title"),
            "tweaks.fast_send_copy_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.disable_ndu_title"),
            "tweaks.disable_ndu_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.disable_ndu_side_effect"),
            "tweaks.disable_ndu_side_effect"
        );

        rust_i18n::set_locale("en");
        assert_ne!(rust_i18n::t!("tweaks.bbr2_title"), "tweaks.bbr2_title");
        assert_ne!(rust_i18n::t!("tweaks.rss_title"), "tweaks.rss_title");
        assert_ne!(
            rust_i18n::t!("tweaks.fast_send_copy_title"),
            "tweaks.fast_send_copy_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.disable_ndu_title"),
            "tweaks.disable_ndu_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.disable_ndu_side_effect"),
            "tweaks.disable_ndu_side_effect"
        );
    }

    #[test]
    fn test_input_tweaks_i18n_keys() {
        rust_i18n::set_locale("ru");
        assert_ne!(
            rust_i18n::t!("tweaks.disable_mouse_acceleration_title"),
            "tweaks.disable_mouse_acceleration_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.csrss_priority_title"),
            "tweaks.csrss_priority_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_optimization_title"),
            "tweaks.ctf_optimization_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_standard"),
            "tweaks.ctf_preset_standard"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_mild"),
            "tweaks.ctf_preset_mild"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_aggressive"),
            "tweaks.ctf_preset_aggressive"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.snapkey_title"),
            "tweaks.snapkey_title"
        );
        assert_ne!(rust_i18n::t!("tweaks.snapkey_wasd"), "tweaks.snapkey_wasd");

        rust_i18n::set_locale("en");
        assert_ne!(
            rust_i18n::t!("tweaks.disable_mouse_acceleration_title"),
            "tweaks.disable_mouse_acceleration_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.csrss_priority_title"),
            "tweaks.csrss_priority_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_optimization_title"),
            "tweaks.ctf_optimization_title"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_standard"),
            "tweaks.ctf_preset_standard"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_mild"),
            "tweaks.ctf_preset_mild"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.ctf_preset_aggressive"),
            "tweaks.ctf_preset_aggressive"
        );
        assert_ne!(
            rust_i18n::t!("tweaks.snapkey_title"),
            "tweaks.snapkey_title"
        );
        assert_ne!(rust_i18n::t!("tweaks.snapkey_wasd"), "tweaks.snapkey_wasd");
    }
}
