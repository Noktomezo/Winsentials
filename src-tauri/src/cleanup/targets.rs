use super::CleanupTarget;

pub(super) const WINDOWS_TEMP_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "user_temp",
        name: "User Temp",
        path: "{TEMP}",
    },
    CleanupTarget {
        id: "local_temp",
        name: "Local AppData Temp",
        path: "{LOCALAPPDATA}\\Temp",
    },
    CleanupTarget {
        id: "windows_temp",
        name: "Windows Temp",
        path: "{WINDIR}\\Temp",
    },
    CleanupTarget {
        id: "root_temp",
        name: "Root Temp",
        path: "C:\\Temp",
    },
    CleanupTarget {
        id: "prefetch",
        name: "Windows Prefetch",
        path: "{WINDIR}\\Prefetch",
    },
    CleanupTarget {
        id: "inet_cache",
        name: "Windows INetCache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\INetCache",
    },
    CleanupTarget {
        id: "delivery_optimization_cache",
        name: "Delivery Optimization Cache",
        path: "{WINDIR}\\SoftwareDistribution\\Download",
    },
];

pub(super) const THUMBNAIL_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "explorer_thumbcache",
        name: "Explorer Thumbnail Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Explorer\\thumbcache_*.db",
    },
    CleanupTarget {
        id: "explorer_iconcache",
        name: "Explorer Icon Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Explorer\\iconcache_*.db",
    },
    CleanupTarget {
        id: "iconcache_db",
        name: "Icon Cache Database",
        path: "{LOCALAPPDATA}\\IconCache.db",
    },
];

pub(super) const BROWSER_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "chrome_cache",
        name: "Google Chrome Cache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "chrome_code_cache",
        name: "Google Chrome Code Cache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\Code Cache",
    },
    CleanupTarget {
        id: "chrome_gpu_cache",
        name: "Google Chrome GPUCache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\GPUCache",
    },
    CleanupTarget {
        id: "edge_cache",
        name: "Microsoft Edge Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "edge_code_cache",
        name: "Microsoft Edge Code Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\Code Cache",
    },
    CleanupTarget {
        id: "edge_gpu_cache",
        name: "Microsoft Edge GPUCache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\GPUCache",
    },
    CleanupTarget {
        id: "brave_cache",
        name: "Brave Cache",
        path: "{LOCALAPPDATA}\\BraveSoftware\\Brave-Browser\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "vivaldi_cache",
        name: "Vivaldi Cache",
        path: "{LOCALAPPDATA}\\Vivaldi\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "opera_cache",
        name: "Opera Cache",
        path: "{APPDATA}\\Opera Software\\Opera Stable\\Cache",
    },
    CleanupTarget {
        id: "opera_gx_cache",
        name: "Opera GX Cache",
        path: "{APPDATA}\\Opera Software\\Opera GX Stable\\Cache",
    },
    CleanupTarget {
        id: "firefox_cache",
        name: "Mozilla Firefox Cache",
        path: "{LOCALAPPDATA}\\Mozilla\\Firefox\\Profiles\\*\\cache2",
    },
    CleanupTarget {
        id: "yandex_cache",
        name: "Yandex Browser Cache",
        path: "{LOCALAPPDATA}\\Yandex\\YandexBrowser\\User Data\\*\\Cache",
    },
];

pub(super) const DRIVER_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "direct3d_shader_cache",
        name: "Direct3D Shader Cache",
        path: "{LOCALAPPDATA}\\D3DSCache",
    },
    CleanupTarget {
        id: "amd_dx_cache",
        name: "AMD DirectX Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\DxCache",
    },
    CleanupTarget {
        id: "amd_dxc_cache",
        name: "AMD DXC Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\DxcCache",
    },
    CleanupTarget {
        id: "amd_vk_cache",
        name: "AMD Vulkan Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\VkCache",
    },
    CleanupTarget {
        id: "amd_legacy_dx_cache",
        name: "AMD Legacy Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\AMD\\DxCache",
    },
    CleanupTarget {
        id: "nvidia_dx_cache",
        name: "NVIDIA DirectX Shader Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\DXCache",
    },
    CleanupTarget {
        id: "nvidia_gl_cache",
        name: "NVIDIA OpenGL Shader Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\GLCache",
    },
    CleanupTarget {
        id: "nvidia_per_driver_dx_cache",
        name: "NVIDIA Per-Driver DirectX Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\NVIDIA\\PerDriverVersion\\DXCache",
    },
    CleanupTarget {
        id: "nvidia_per_driver_gl_cache",
        name: "NVIDIA Per-Driver OpenGL Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\NVIDIA\\PerDriverVersion\\GLCache",
    },
    CleanupTarget {
        id: "nvidia_compute_cache",
        name: "NVIDIA Compute Cache",
        path: "{APPDATA}\\NVIDIA\\ComputeCache",
    },
    CleanupTarget {
        id: "nvidia_downloader_cache",
        name: "NVIDIA Driver Download Cache",
        path: "{PROGRAMDATA}\\NVIDIA Corporation\\Downloader",
    },
    CleanupTarget {
        id: "nvidia_nv_cache",
        name: "NVIDIA NV Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\NvBackend\\Cache",
    },
    CleanupTarget {
        id: "nvidia_physx_cache",
        name: "NVIDIA PhysX Cache",
        path: "{LOCALAPPDATA}\\NVIDIA Corporation\\PhysX\\Cache",
    },
    CleanupTarget {
        id: "intel_shader_cache",
        name: "Intel Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\Intel\\ShaderCache",
    },
    CleanupTarget {
        id: "intel_local_shader_cache",
        name: "Intel Local Shader Cache",
        path: "{LOCALAPPDATA}\\Intel\\ShaderCache",
    },
    CleanupTarget {
        id: "amd_installer_leftovers",
        name: "AMD Driver Installer Leftovers",
        path: "C:\\AMD",
    },
    CleanupTarget {
        id: "amd_temp_leftovers",
        name: "AMD Driver Temp Leftovers",
        path: "{TEMP}\\AMD",
    },
    CleanupTarget {
        id: "amd_tmp_leftovers",
        name: "AMD Driver TMP Leftovers",
        path: "{TMP}\\AMD",
    },
    CleanupTarget {
        id: "amd_local_temp_leftovers",
        name: "AMD Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\AMD",
    },
    CleanupTarget {
        id: "nvidia_temp_leftovers",
        name: "NVIDIA Driver Temp Leftovers",
        path: "{TEMP}\\NVIDIA",
    },
    CleanupTarget {
        id: "nvidia_tmp_leftovers",
        name: "NVIDIA Driver TMP Leftovers",
        path: "{TMP}\\NVIDIA",
    },
    CleanupTarget {
        id: "nvidia_local_temp_leftovers",
        name: "NVIDIA Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\NVIDIA",
    },
    CleanupTarget {
        id: "intel_temp_leftovers",
        name: "Intel Driver Temp Leftovers",
        path: "{TEMP}\\Intel",
    },
    CleanupTarget {
        id: "intel_tmp_leftovers",
        name: "Intel Driver TMP Leftovers",
        path: "{TMP}\\Intel",
    },
    CleanupTarget {
        id: "intel_local_temp_leftovers",
        name: "Intel Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\Intel",
    },
];

pub(super) const GAME_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "steam_htmlcache",
        name: "Steam HTML Cache",
        path: "{LOCALAPPDATA}\\Steam\\htmlcache",
    },
    CleanupTarget {
        id: "steam_shader_cache",
        name: "Steam Shader Cache",
        path: "{PROGRAMFILES_X86}\\Steam\\steamapps\\shadercache",
    },
    CleanupTarget {
        id: "steam_downloading_cache",
        name: "Steam Download Cache",
        path: "{PROGRAMFILES_X86}\\Steam\\steamapps\\downloading",
    },
    CleanupTarget {
        id: "epic_webcache",
        name: "Epic Games Launcher Web Cache",
        path: "{LOCALAPPDATA}\\EpicGamesLauncher\\Saved\\webcache*",
    },
    CleanupTarget {
        id: "ea_desktop_cache",
        name: "EA Desktop Cache",
        path: "{LOCALAPPDATA}\\Electronic Arts\\EA Desktop\\cache",
    },
    CleanupTarget {
        id: "origin_cache",
        name: "Origin Cache",
        path: "{PROGRAMDATA}\\Origin\\DownloadCache",
    },
];

pub(super) const WINDOWS_LOGS_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "windows_cbs_logs",
        name: "Windows CBS Logs",
        path: "{WINDIR}\\Logs\\CBS",
    },
    CleanupTarget {
        id: "windows_dism_logs",
        name: "Windows DISM Logs",
        path: "{WINDIR}\\Logs\\DISM",
    },
    CleanupTarget {
        id: "windows_setup_logs",
        name: "Windows Setup Logs",
        path: "{WINDIR}\\Panther",
    },
    CleanupTarget {
        id: "windows_logfiles",
        name: "Windows LogFiles",
        path: "{WINDIR}\\System32\\LogFiles",
    },
    CleanupTarget {
        id: "setupapi_logs",
        name: "SetupAPI Logs",
        path: "{WINDIR}\\INF\\setupapi*.log",
    },
];

pub(super) const SYSTEM_ERROR_REPORTS_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "windows_error_reports_archive",
        name: "Windows Error Reports Archive",
        path: "{PROGRAMDATA}\\Microsoft\\Windows\\WER\\ReportArchive",
    },
    CleanupTarget {
        id: "windows_error_reports_queue",
        name: "Windows Error Reports Queue",
        path: "{PROGRAMDATA}\\Microsoft\\Windows\\WER\\ReportQueue",
    },
    CleanupTarget {
        id: "user_error_reports_archive",
        name: "User Error Reports Archive",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\WER\\ReportArchive",
    },
    CleanupTarget {
        id: "user_error_reports_queue",
        name: "User Error Reports Queue",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\WER\\ReportQueue",
    },
    CleanupTarget {
        id: "minidumps",
        name: "Windows Minidumps",
        path: "{WINDIR}\\Minidump",
    },
    CleanupTarget {
        id: "memory_dump",
        name: "Windows Memory Dump",
        path: "{WINDIR}\\MEMORY.DMP",
    },
];

pub(super) const APP_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "microsoft_store_cache",
        name: "Microsoft Store Cache",
        path: "{LOCALAPPDATA}\\Packages\\Microsoft.WindowsStore_*\\LocalCache",
    },
    CleanupTarget {
        id: "windows_live_tiles_cache",
        name: "Windows Live Tiles Cache",
        path: "{LOCALAPPDATA}\\Packages\\Microsoft.Windows.StartMenuExperienceHost_*\\LocalState\\Cache",
    },
    CleanupTarget {
        id: "windows_notifications_cache",
        name: "Windows Notifications Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Notifications",
    },
    CleanupTarget {
        id: "onedrive_logs",
        name: "OneDrive Logs",
        path: "{LOCALAPPDATA}\\Microsoft\\OneDrive\\logs",
    },
    CleanupTarget {
        id: "office_file_cache",
        name: "Microsoft Office File Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Office\\16.0\\OfficeFileCache",
    },
    CleanupTarget {
        id: "teams_cache",
        name: "Microsoft Teams Cache",
        path: "{APPDATA}\\Microsoft\\Teams\\Cache",
    },
    CleanupTarget {
        id: "discord_cache",
        name: "Discord Cache",
        path: "{APPDATA}\\discord\\Cache",
    },
];
