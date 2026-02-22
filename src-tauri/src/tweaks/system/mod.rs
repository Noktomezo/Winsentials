mod boot_optimize;
mod disable_background_apps;
mod disable_fso;
mod disable_gpu_energy_drv;
mod disable_hibernation;
mod disable_superfetch;
mod game_mode;
mod prefetcher;
mod shutdown_timeouts;
mod win32_priority_separation;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(win32_priority_separation::Win32PrioritySeparationTweak::new()),
    Box::new(boot_optimize::BootOptimizeFunctionTweak::new()),
    Box::new(prefetcher::PrefetcherTweak::new()),
    Box::new(game_mode::GameModeTweak::new()),
    Box::new(disable_fso::DisableFullsreenOptimizationsTweak::new()),
    Box::new(disable_gpu_energy_drv::DisableGpuEnergyDrvTweak::new()),
    Box::new(disable_hibernation::DisableHibernationTweak::new()),
    Box::new(shutdown_timeouts::ShutdownTimeoutsTweak::new()),
    Box::new(disable_background_apps::DisableBackgroundAppsTweak::new()),
    Box::new(disable_superfetch::DisableSuperfetchTweak::new()),
  ]
}
