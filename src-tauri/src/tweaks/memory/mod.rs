mod clear_pagefile_shutdown;
mod disable_fth;
mod disable_page_combining;
mod disable_paging_executive;
mod heap_decommit;
mod large_system_cache;
mod svchost_split_threshold;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(disable_paging_executive::DisablePagingExecutiveTweak::new()),
    Box::new(clear_pagefile_shutdown::ClearPageFileShutdownTweak::new()),
    Box::new(large_system_cache::LargeSystemCacheTweak::new()),
    Box::new(disable_page_combining::DisablePageCombiningTweak::new()),
    Box::new(svchost_split_threshold::SvcHostSplitThresholdTweak::new()),
    Box::new(heap_decommit::HeapDecommitTweak::new()),
    Box::new(disable_fth::DisableFthTweak::new()),
  ]
}
