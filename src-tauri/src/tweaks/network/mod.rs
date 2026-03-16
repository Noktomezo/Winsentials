pub mod disable_qos_bandwidth_limit;
pub mod enable_bbr2;
pub mod enable_network_offloading;

use crate::tweaks::Tweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(enable_network_offloading::EnableNetworkOffloadingTweak::new()),
        Box::new(disable_qos_bandwidth_limit::DisableQosBandwidthLimitTweak::new()),
        Box::new(enable_bbr2::EnableBbr2Tweak::new()),
    ]
}
