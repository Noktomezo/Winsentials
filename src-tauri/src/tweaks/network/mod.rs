pub mod disable_ncsi_active_probing;
pub mod disable_ndu;
pub mod disable_qos_bandwidth_limit;
pub mod enable_bbr2;
pub mod enable_network_offloading;
pub mod fast_udp_optimization;
pub mod native;

use crate::tweaks::Tweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(enable_network_offloading::EnableNetworkOffloadingTweak::new()),
        Box::new(disable_ndu::DisableNduTweak::new()),
        Box::new(disable_ncsi_active_probing::DisableNcsiActiveProbingTweak::new()),
        Box::new(disable_qos_bandwidth_limit::DisableQosBandwidthLimitTweak::new()),
        Box::new(fast_udp_optimization::FastUdpOptimizationTweak::new()),
        Box::new(enable_bbr2::EnableBbr2Tweak::new()),
    ]
}
