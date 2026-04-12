use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, SC_HANDLE,
    SC_MANAGER_CONNECT, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS, SERVICE_STATUS, SERVICE_STOP,
};
use windows::core::{Error as WindowsError, PCWSTR};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const MIN_WINDOWS_10_1809_BUILD: u32 = 17763;

const DEFAULT_ALLOW_TELEMETRY: u32 = 1;
const DISABLED_ALLOW_TELEMETRY: u32 = 0;
const DEFAULT_RESTRICT_DEVICE_METADATA: u32 = 0;
const ENABLED_RESTRICT_DEVICE_METADATA: u32 = 1;

const DIAGTRACK_DEFAULT_START: u32 = 2;
const DMWAPPUSHSERVICE_DEFAULT_START: u32 = 3;
const DISABLED_SERVICE_START: u32 = 4;

const SERVICE_DOES_NOT_EXIST_HRESULT: i32 = -2147023836;
const SERVICE_NOT_ACTIVE_HRESULT: i32 = -2147023834;

const POLICY_DATA_COLLECTION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
};

const CURRENT_DATA_COLLECTION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection",
};

const DIAGTRACK_SERVICE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\DiagTrack",
};

const DMWAPPUSHSERVICE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\dmwappushservice",
};

pub struct DisableWindowsTelemetryTweak {
    meta: TweakMeta,
}

struct TelemetryState {
    policy_allow_telemetry: u32,
    current_allow_telemetry: u32,
    policy_restrict_metadata: u32,
    current_restrict_metadata: u32,
    diagtrack_start: u32,
    dmwappushservice_start: u32,
}

struct ServiceHandle(SC_HANDLE);

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        // Best-effort cleanup for handles opened via the Service Control Manager.
        unsafe {
            let _ = CloseServiceHandle(self.0);
        }
    }
}

impl Default for DisableWindowsTelemetryTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableWindowsTelemetryTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_windows_telemetry".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableWindowsTelemetry.name".into(),
                short_description: "privacy.tweaks.disableWindowsTelemetry.shortDescription".into(),
                detail_description: "privacy.tweaks.disableWindowsTelemetry.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableWindowsTelemetry.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_1809_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_dword_or_default(key: &RegKey, name: &str, default: u32) -> Result<u32, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn set_registry_values(
        &self,
        allow_telemetry: u32,
        restrict_metadata: u32,
    ) -> Result<(), AppError> {
        POLICY_DATA_COLLECTION_KEY.set_dword("AllowTelemetry", allow_telemetry)?;
        CURRENT_DATA_COLLECTION_KEY.set_dword("AllowTelemetry", allow_telemetry)?;
        POLICY_DATA_COLLECTION_KEY.set_dword("RestrictDeviceMetadata", restrict_metadata)?;
        CURRENT_DATA_COLLECTION_KEY.set_dword("RestrictDeviceMetadata", restrict_metadata)
    }

    fn set_service_start_values(
        &self,
        diagtrack_start: u32,
        dmwappushservice_start: u32,
    ) -> Result<(), AppError> {
        DIAGTRACK_SERVICE_KEY.set_dword("Start", diagtrack_start)?;
        DMWAPPUSHSERVICE_KEY.set_dword("Start", dmwappushservice_start)
    }

    fn stop_service(&self, service_name: &str) -> Result<(), AppError> {
        let service_manager = ServiceHandle(unsafe {
            OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)?
        });
        let service_name_wide = wide(service_name);
        let service = match unsafe {
            OpenServiceW(
                service_manager.0,
                PCWSTR(service_name_wide.as_ptr()),
                SERVICE_STOP | SERVICE_QUERY_STATUS,
            )
        } {
            Ok(handle) => ServiceHandle(handle),
            Err(error) if is_ignorable_service_error(&error) => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        let mut status = SERVICE_STATUS::default();

        match unsafe { ControlService(service.0, SERVICE_CONTROL_STOP, &mut status) } {
            Ok(()) => Ok(()),
            Err(error) if is_ignorable_service_error(&error) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn read_state(&self) -> Result<TelemetryState, AppError> {
        Ok(TelemetryState {
            policy_allow_telemetry: Self::read_dword_or_default(
                &POLICY_DATA_COLLECTION_KEY,
                "AllowTelemetry",
                DEFAULT_ALLOW_TELEMETRY,
            )?,
            current_allow_telemetry: Self::read_dword_or_default(
                &CURRENT_DATA_COLLECTION_KEY,
                "AllowTelemetry",
                DEFAULT_ALLOW_TELEMETRY,
            )?,
            policy_restrict_metadata: Self::read_dword_or_default(
                &POLICY_DATA_COLLECTION_KEY,
                "RestrictDeviceMetadata",
                DEFAULT_RESTRICT_DEVICE_METADATA,
            )?,
            current_restrict_metadata: Self::read_dword_or_default(
                &CURRENT_DATA_COLLECTION_KEY,
                "RestrictDeviceMetadata",
                DEFAULT_RESTRICT_DEVICE_METADATA,
            )?,
            diagtrack_start: Self::read_dword_or_default(
                &DIAGTRACK_SERVICE_KEY,
                "Start",
                DIAGTRACK_DEFAULT_START,
            )?,
            dmwappushservice_start: Self::read_dword_or_default(
                &DMWAPPUSHSERVICE_KEY,
                "Start",
                DMWAPPUSHSERVICE_DEFAULT_START,
            )?,
        })
    }

    fn is_enabled(state: &TelemetryState) -> bool {
        state.policy_allow_telemetry == DISABLED_ALLOW_TELEMETRY
            && state.current_allow_telemetry == DISABLED_ALLOW_TELEMETRY
            && state.policy_restrict_metadata == ENABLED_RESTRICT_DEVICE_METADATA
            && state.current_restrict_metadata == ENABLED_RESTRICT_DEVICE_METADATA
            && state.diagtrack_start == DISABLED_SERVICE_START
            && state.dmwappushservice_start == DISABLED_SERVICE_START
    }

    fn is_default(state: &TelemetryState) -> bool {
        state.policy_allow_telemetry == DEFAULT_ALLOW_TELEMETRY
            && state.current_allow_telemetry == DEFAULT_ALLOW_TELEMETRY
            && state.policy_restrict_metadata == DEFAULT_RESTRICT_DEVICE_METADATA
            && state.current_restrict_metadata == DEFAULT_RESTRICT_DEVICE_METADATA
            && state.diagtrack_start == DIAGTRACK_DEFAULT_START
            && state.dmwappushservice_start == DMWAPPUSHSERVICE_DEFAULT_START
    }
}

impl Tweak for DisableWindowsTelemetryTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                self.set_registry_values(
                    DISABLED_ALLOW_TELEMETRY,
                    ENABLED_RESTRICT_DEVICE_METADATA,
                )?;
                self.set_service_start_values(DISABLED_SERVICE_START, DISABLED_SERVICE_START)?;
                self.stop_service("DiagTrack")?;
                self.stop_service("dmwappushservice")
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.set_registry_values(DEFAULT_ALLOW_TELEMETRY, DEFAULT_RESTRICT_DEVICE_METADATA)?;
        self.set_service_start_values(DIAGTRACK_DEFAULT_START, DMWAPPUSHSERVICE_DEFAULT_START)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let state = self.read_state()?;
        let is_enabled = Self::is_enabled(&state);
        let is_default = Self::is_default(&state);

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                CUSTOM_VALUE.into()
            },
            is_default,
        })
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn is_ignorable_service_error(error: &WindowsError) -> bool {
    matches!(
        error.code().0,
        SERVICE_DOES_NOT_EXIST_HRESULT | SERVICE_NOT_ACTIVE_HRESULT
    )
}
