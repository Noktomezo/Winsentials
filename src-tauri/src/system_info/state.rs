use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::system_info::types::{LiveSystemInfo, StaticSystemInfo};

pub struct SystemInfoState {
    pub system: Mutex<System>,
    pub networks: Mutex<Networks>,
    pub prev_net: Mutex<Option<PreviousNetSnapshot>>,
    pub static_cache: Mutex<Option<StaticSystemInfo>>,
    pub live_cache: Mutex<Option<LiveSystemInfo>>,
}

#[derive(Debug, Clone)]
pub struct PreviousNetSnapshot {
    pub captured_at: Instant,
    pub totals: HashMap<String, (u64, u64)>,
}

impl Default for SystemInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoState {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let networks = Networks::new_with_refreshed_list();
        Self {
            system: Mutex::new(system),
            networks: Mutex::new(networks),
            prev_net: Mutex::new(None),
            static_cache: Mutex::new(None),
            live_cache: Mutex::new(None),
        }
    }
}
