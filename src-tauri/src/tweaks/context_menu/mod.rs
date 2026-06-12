pub mod create_symbolic_link;
pub mod take_ownership;

use crate::tweaks::Tweak;

use self::create_symbolic_link::CreateSymbolicLinkContextMenuTweak;
use self::take_ownership::TakeOwnershipTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(CreateSymbolicLinkContextMenuTweak::new()),
        Box::new(TakeOwnershipTweak::new()),
    ]
}
