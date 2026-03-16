pub mod disable_8dot3_name_creation;
pub mod disable_recent_items_and_frequent_places;
pub mod open_explorer_to_this_pc;
pub mod unlock_lock_screen_timeout_setting;

use crate::tweaks::Tweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(open_explorer_to_this_pc::OpenExplorerToThisPcTweak::new()),
        Box::new(disable_8dot3_name_creation::Disable8dot3NameCreationTweak::new()),
        Box::new(
            disable_recent_items_and_frequent_places::DisableRecentItemsAndFrequentPlacesTweak::new(
            ),
        ),
        Box::new(unlock_lock_screen_timeout_setting::UnlockLockScreenTimeoutSettingTweak::new()),
    ]
}
