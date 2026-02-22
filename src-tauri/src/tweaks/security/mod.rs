mod core_isolation;
mod sehop;
mod spectre_meltdown;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(core_isolation::CoreIsolationTweak::new()),
    Box::new(spectre_meltdown::SpectreMeltdownTweak::new()),
    Box::new(sehop::SehopTweak::new()),
  ]
}
