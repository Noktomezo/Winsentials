#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AppRoute {
    #[default]
    Dashboard,
    CpuDetail,
    RamDetail,
    DiskDetail(usize),
    NetworkDetail(u32),
    GpuDetail(usize),
    ContextMenu,
    Explorer,
    Interface,
    Input,
    Settings,
}

impl AppRoute {
    #[must_use]
    pub fn title(self) -> String {
        match self {
            Self::Dashboard => rust_i18n::t!("nav.dashboard").to_string(),
            Self::CpuDetail => rust_i18n::t!("cpu_detail.title").to_string(),
            Self::RamDetail => rust_i18n::t!("ram_detail.title").to_string(),
            Self::DiskDetail(id) => format!("{} {id}", rust_i18n::t!("telemetry.disk")),
            Self::NetworkDetail(_) => rust_i18n::t!("telemetry.network").to_string(),
            Self::GpuDetail(id) => format!("{} {id}", rust_i18n::t!("telemetry.gpu")),
            Self::ContextMenu => rust_i18n::t!("nav.context_menu").to_string(),
            Self::Explorer => rust_i18n::t!("nav.explorer").to_string(),
            Self::Interface => rust_i18n::t!("nav.interface").to_string(),
            Self::Input => rust_i18n::t!("nav.input").to_string(),
            Self::Settings => rust_i18n::t!("nav.settings").to_string(),
        }
    }

    #[must_use]
    pub fn english_name(self) -> String {
        match self {
            Self::Dashboard => "Dashboard".to_string(),
            Self::CpuDetail => "Processor (CPU)".to_string(),
            Self::RamDetail => "Memory (RAM)".to_string(),
            Self::DiskDetail(id) => format!("Disk {id}"),
            Self::NetworkDetail(_) => "Network".to_string(),
            Self::GpuDetail(id) => format!("GPU {id}"),
            Self::ContextMenu => "Context Menu".to_string(),
            Self::Explorer => "File Explorer".to_string(),
            Self::Interface => "Interface".to_string(),
            Self::Input => "Input".to_string(),
            Self::Settings => "Settings".to_string(),
        }
    }

    #[must_use]
    pub fn breadcrumb_english(self) -> String {
        match self {
            Self::CpuDetail
            | Self::RamDetail
            | Self::DiskDetail(_)
            | Self::NetworkDetail(_)
            | Self::GpuDetail(_) => {
                format!("Dashboard > {}", self.english_name())
            }
            _ => self.english_name(),
        }
    }

    #[must_use]
    pub fn description(self) -> String {
        match self {
            Self::Dashboard => rust_i18n::t!("nav.dashboard_desc").to_string(),
            Self::CpuDetail => rust_i18n::t!("cpu_detail.desc").to_string(),
            Self::RamDetail => rust_i18n::t!("ram_detail.desc").to_string(),
            Self::DiskDetail(_) => rust_i18n::t!("disk_detail.desc").to_string(),
            Self::NetworkDetail(_) => rust_i18n::t!("network_detail.desc").to_string(),
            Self::GpuDetail(_) => rust_i18n::t!("gpu_detail.desc").to_string(),
            Self::ContextMenu => rust_i18n::t!("nav.context_menu_desc").to_string(),
            Self::Explorer => rust_i18n::t!("nav.explorer_desc").to_string(),
            Self::Interface => rust_i18n::t!("nav.interface_desc").to_string(),
            Self::Input => rust_i18n::t!("nav.input_desc").to_string(),
            Self::Settings => rust_i18n::t!("nav.settings_desc").to_string(),
        }
    }

    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "icons/house.svg",
            Self::CpuDetail => "icons/cpu.svg",
            Self::RamDetail => "icons/memory-stick.svg",
            Self::DiskDetail(_) => "icons/hard-drive.svg",
            Self::NetworkDetail(_) => "icons/network.svg",
            Self::GpuDetail(_) => "icons/circuit-board.svg",
            Self::ContextMenu => "icons/square-menu.svg",
            Self::Explorer => "icons/folder.svg",
            Self::Interface => "icons/layout-grid.svg",
            Self::Input => "icons/mouse.svg",
            Self::Settings => "icons/settings.svg",
        }
    }

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Dashboard => "nav_dashboard",
            Self::CpuDetail => "nav_cpu_detail",
            Self::RamDetail => "nav_ram_detail",
            Self::DiskDetail(_) => "nav_disk_detail",
            Self::NetworkDetail(_) => "nav_network_detail",
            Self::GpuDetail(_) => "nav_gpu_detail",
            Self::ContextMenu => "nav_context_menu",
            Self::Explorer => "nav_explorer",
            Self::Interface => "nav_interface",
            Self::Input => "nav_input",
            Self::Settings => "nav_settings",
        }
    }

    pub const TOP_NAV: [Self; 5] = [
        Self::Dashboard,
        Self::ContextMenu,
        Self::Explorer,
        Self::Interface,
        Self::Input,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_english_name() {
        assert_eq!(AppRoute::Dashboard.english_name(), "Dashboard");
        assert_eq!(AppRoute::CpuDetail.english_name(), "Processor (CPU)");
        assert_eq!(AppRoute::RamDetail.english_name(), "Memory (RAM)");
        assert_eq!(AppRoute::DiskDetail(0).english_name(), "Disk 0");
        assert_eq!(AppRoute::NetworkDetail(1).english_name(), "Network");
        assert_eq!(AppRoute::GpuDetail(0).english_name(), "GPU 0");
        assert_eq!(AppRoute::ContextMenu.english_name(), "Context Menu");
        assert_eq!(AppRoute::Explorer.english_name(), "File Explorer");
        assert_eq!(AppRoute::Interface.english_name(), "Interface");
        assert_eq!(AppRoute::Input.english_name(), "Input");
        assert_eq!(AppRoute::Settings.english_name(), "Settings");
    }

    #[test]
    fn test_route_breadcrumb_english() {
        assert_eq!(AppRoute::Dashboard.breadcrumb_english(), "Dashboard");
        assert_eq!(
            AppRoute::GpuDetail(0).breadcrumb_english(),
            "Dashboard > GPU 0"
        );
        assert_eq!(
            AppRoute::CpuDetail.breadcrumb_english(),
            "Dashboard > Processor (CPU)"
        );
        assert_eq!(AppRoute::ContextMenu.breadcrumb_english(), "Context Menu");
        assert_eq!(AppRoute::Input.breadcrumb_english(), "Input");
    }
}
