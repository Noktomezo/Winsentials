use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CleanupCategory {
    Windows,
    Browsers,
    Applications,
    Development,
    Games,
    Media,
    Devices,
}

impl CleanupCategory {
    pub const ALL: [Self; 7] = [
        Self::Windows,
        Self::Browsers,
        Self::Applications,
        Self::Development,
        Self::Games,
        Self::Media,
        Self::Devices,
    ];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Browsers => "browsers",
            Self::Applications => "applications",
            Self::Development => "development",
            Self::Games => "games",
            Self::Media => "media",
            Self::Devices => "devices",
        }
    }

    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Windows => "icons/monitor-cog.svg",
            Self::Browsers => "icons/globe.svg",
            Self::Applications => "icons/app-window.svg",
            Self::Development => "icons/code-xml.svg",
            Self::Games => "icons/gamepad-2.svg",
            Self::Media => "icons/video.svg",
            Self::Devices => "icons/usb.svg",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CleanupPath {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub struct CleanupTarget {
    pub id: String,
    pub name: String,
    pub category: CleanupCategory,
    pub paths: Vec<CleanupPath>,
    pub(crate) prune_roots: Vec<PathBuf>,
    pub device_instance_id: Option<String>,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupSnapshot {
    pub targets: Vec<CleanupTarget>,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupReport {
    pub removed_bytes: u64,
    pub removed_paths: usize,
    pub failures: usize,
}

#[derive(Debug, Error)]
pub enum CleanupError {
    #[error("cleanup path is outside the allowed roots: {0}")]
    UnsafePath(PathBuf),
    #[error("could not remove {path}")]
    Remove {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not remove unused device {0}")]
    DeviceRemoval(String),
}

#[derive(Clone, Debug, Default)]
pub struct CleanupState {
    pub snapshot: CleanupSnapshot,
    pub selected: HashSet<String>,
    pub expanded: Option<CleanupCategory>,
    pub scanning: bool,
    pub cleaning: bool,
    pub scanned_once: bool,
}

impl CleanupState {
    pub fn apply_snapshot(&mut self, snapshot: CleanupSnapshot) {
        let available = snapshot
            .targets
            .iter()
            .map(|target| target.id.as_str())
            .collect::<HashSet<_>>();
        self.selected.retain(|id| available.contains(id.as_str()));
        self.snapshot = snapshot;
        self.scanning = false;
        self.scanned_once = true;
    }

    pub fn toggle_target(&mut self, id: &str) {
        if !self.selected.remove(id) {
            self.selected.insert(id.to_owned());
        }
    }

    pub fn toggle_category(&mut self, category: CleanupCategory) {
        let ids = self
            .snapshot
            .targets
            .iter()
            .filter(|target| target.category == category)
            .map(|target| target.id.clone())
            .collect::<Vec<_>>();
        let select = ids.iter().any(|id| !self.selected.contains(id));
        for id in ids {
            if select {
                self.selected.insert(id);
            } else {
                self.selected.remove(&id);
            }
        }
    }

    pub fn toggle_all(&mut self) {
        if self.selected.len() == self.snapshot.targets.len() {
            self.selected.clear();
        } else {
            self.selected = self
                .snapshot
                .targets
                .iter()
                .map(|target| target.id.clone())
                .collect();
        }
    }

    #[must_use]
    pub fn selected_totals(&self) -> (usize, u64) {
        self.snapshot
            .targets
            .iter()
            .filter(|target| self.selected.contains(&target.id))
            .fold((0, 0), |(count, bytes), target| {
                (count + 1, bytes + target.bytes)
            })
    }
}

#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["Б", "КБ", "МБ", "ГБ"];
    let mut value = bytes;
    let mut unit = 0;
    let mut remainder = 0;
    while value >= 1024 && unit < UNITS.len() - 1 {
        remainder = value % 1024;
        value /= 1024;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} {}", UNITS[unit])
    } else {
        format!(
            "{value}.{} {}",
            remainder.saturating_mul(10) / 1024,
            UNITS[unit]
        )
    }
}