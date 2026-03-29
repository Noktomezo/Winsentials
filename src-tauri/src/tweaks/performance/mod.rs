pub mod disable_fault_tolerant_heap;
pub mod optimize_mmcss;

use crate::tweaks::Tweak;

use self::disable_fault_tolerant_heap::DisableFaultTolerantHeapTweak;
use self::optimize_mmcss::OptimizeMmcssTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(DisableFaultTolerantHeapTweak::new()),
        Box::new(OptimizeMmcssTweak::new()),
    ]
}
