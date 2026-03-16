pub mod disable_open_file_warning;
pub mod disable_security_center_notifications;
pub mod disable_user_account_control;

use crate::tweaks::Tweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(
            disable_security_center_notifications::DisableSecurityCenterNotificationsTweak::new(),
        ),
        Box::new(disable_open_file_warning::DisableOpenFileWarningTweak::new()),
        Box::new(disable_user_account_control::DisableUserAccountControlTweak::new()),
    ]
}
