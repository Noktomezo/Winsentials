use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub prune_roots: Vec<PathBuf>,
    pub device_instance_id: Option<String>,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupSnapshot {
    pub targets: Vec<CleanupTarget>,
}

#[derive(Clone, Copy, Debug, Default)]
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
    pub snapshot: Arc<CleanupSnapshot>,
    pub selected: HashSet<String>,
    pub expanded: Option<CleanupCategory>,
    pub scanning_categories: HashSet<CleanupCategory>,
    pub cleaning_categories: HashSet<CleanupCategory>,
    pub recently_cleaned: HashMap<CleanupCategory, Instant>,
    pub scanned_categories: HashSet<CleanupCategory>,
    pub scanning: bool,
    pub cleaning: bool,
    pub scanned_once: bool,
}

impl CleanupState {
    #[must_use]
    pub fn is_category_scanning(&self, category: CleanupCategory) -> bool {
        self.scanning_categories.contains(&category)
    }

    #[must_use]
    pub fn is_category_cleaning(&self, category: CleanupCategory) -> bool {
        self.cleaning_categories.contains(&category)
    }

    #[must_use]
    pub fn is_category_recently_cleaned(&self, category: CleanupCategory) -> bool {
        self.recently_cleaned
            .get(&category)
            .is_some_and(|instant| instant.elapsed() < Duration::from_millis(2000))
    }

    pub fn update_category_targets(
        &mut self,
        category: CleanupCategory,
        new_targets: Vec<CleanupTarget>,
    ) {
        let mut targets: Vec<CleanupTarget> = self
            .snapshot
            .targets
            .iter()
            .filter(|t| t.category != category)
            .cloned()
            .collect();
        targets.extend(new_targets);
        targets.sort_by_key(|t| (t.category.id(), t.name.to_lowercase()));

        let available = targets
            .iter()
            .map(|t| t.id.as_str())
            .collect::<HashSet<_>>();
        self.selected.retain(|id| available.contains(id.as_str()));
        if let Some(exp) = self.expanded {
            if !targets.iter().any(|t| t.category == exp) {
                self.expanded = None;
            }
        }
        self.snapshot = Arc::new(CleanupSnapshot { targets });
        self.scanning_categories.remove(&category);
        self.scanned_categories.insert(category);
        self.scanning = !self.scanning_categories.is_empty();
        self.scanned_once = true;
    }

    pub fn mark_category_cleaned(&mut self, category: CleanupCategory) {
        self.cleaning_categories.remove(&category);
        self.recently_cleaned.insert(category, Instant::now());
        self.cleaning = !self.cleaning_categories.is_empty();
    }

    pub fn apply_snapshot(&mut self, snapshot: CleanupSnapshot) {
        let available = snapshot
            .targets
            .iter()
            .map(|target| target.id.as_str())
            .collect::<HashSet<_>>();
        self.selected.retain(|id| available.contains(id.as_str()));
        if let Some(exp) = self.expanded {
            if !snapshot.targets.iter().any(|t| t.category == exp) {
                self.expanded = None;
            }
        }
        self.snapshot = Arc::new(snapshot);
        self.scanning_categories.clear();
        self.scanned_categories.extend(CleanupCategory::ALL);
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
