mod classic_context_menu;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![Box::new(
    classic_context_menu::ClassicContextMenuTweak::new(),
  )]
}
