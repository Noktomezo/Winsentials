mod dns_cache;
mod network_throttling;
mod tcp_no_delay;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(tcp_no_delay::TcpNoDelayTweak::new()),
    Box::new(dns_cache::DnsCacheTweak::new()),
    Box::new(network_throttling::NetworkThrottlingTweak::new()),
  ]
}
