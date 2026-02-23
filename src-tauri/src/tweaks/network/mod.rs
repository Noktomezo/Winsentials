mod disable_llmnr;
mod disable_nagle;
mod disable_ncsi;
mod disable_qos_reserve;
mod disable_teredo;
mod dns_cache;
mod increase_port_range;
mod network_throttling;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(disable_nagle::DisableNagleTweak::new()),
    Box::new(disable_teredo::DisableTeredoTweak::new()),
    Box::new(disable_llmnr::DisableLlmnrTweak::new()),
    Box::new(disable_qos_reserve::DisableQosReserveTweak::new()),
    Box::new(increase_port_range::IncreasePortRangeTweak::new()),
    Box::new(disable_ncsi::DisableNcsiTweak::new()),
    Box::new(dns_cache::DnsCacheTweak::new()),
    Box::new(network_throttling::NetworkThrottlingTweak::new()),
  ]
}
