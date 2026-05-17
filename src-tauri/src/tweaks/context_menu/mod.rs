pub mod create_symbolic_link;

use crate::tweaks::Tweak;

use self::create_symbolic_link::CreateSymbolicLinkContextMenuTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![Box::new(CreateSymbolicLinkContextMenuTweak::new())]
}
