pub mod disable_fault_tolerant_heap;
pub mod svchost_split_threshold;

use crate::tweaks::Tweak;

use self::disable_fault_tolerant_heap::DisableFaultTolerantHeapTweak;
use self::svchost_split_threshold::SvcHostSplitThresholdTweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(DisableFaultTolerantHeapTweak::new()),
        Box::new(SvcHostSplitThresholdTweak::new()),
    ]
}
