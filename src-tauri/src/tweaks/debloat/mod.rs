pub mod block_razer_auto_install;
pub mod browser_policy;
pub mod disable_copilot;
pub mod remove_microsoft_edge;
pub mod remove_onedrive;

use crate::tweaks::Tweak;

use self::block_razer_auto_install::BlockRazerAutoInstallTweak;
use self::browser_policy::BrowserPolicyDebloatTweak;
use self::disable_copilot::DisableCopilotTweak;
use self::remove_microsoft_edge::RemoveMicrosoftEdgeTweak;
use self::remove_onedrive::RemoveOneDriveTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(BrowserPolicyDebloatTweak::new_edge()),
        Box::new(BrowserPolicyDebloatTweak::new_brave()),
        Box::new(DisableCopilotTweak::new()),
        Box::new(RemoveMicrosoftEdgeTweak::new()),
        Box::new(RemoveOneDriveTweak::new()),
        Box::new(BlockRazerAutoInstallTweak::new()),
    ]
}
