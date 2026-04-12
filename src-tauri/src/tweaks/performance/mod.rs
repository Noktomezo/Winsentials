pub mod configure_kernel_timing_chain;
pub mod disable_game_dvr;
pub mod optimize_mmcss;

use crate::tweaks::Tweak;

use self::configure_kernel_timing_chain::ConfigureKernelTimingChainTweak;
use self::disable_game_dvr::DisableGameDvrTweak;
use self::optimize_mmcss::OptimizeMmcssTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(ConfigureKernelTimingChainTweak::new()),
        Box::new(DisableGameDvrTweak::new()),
        Box::new(OptimizeMmcssTweak::new()),
    ]
}
