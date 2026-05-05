use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RawRegValue, RegKey};
use crate::tweaks::{
    RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakOption, TweakStatus,
};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, SC_HANDLE,
    SC_MANAGER_CONNECT, SERVICE_CONTROL_STOP, SERVICE_QUERY_STATUS, SERVICE_STATUS, SERVICE_STOP,
};
use windows::core::{Error as WindowsError, PCWSTR};
use winreg::enums::RegType::REG_EXPAND_SZ;

const DEFAULT_VALUE: &str = "default";
const SOFT_VALUE: &str = "soft";
const AGGRESSIVE_VALUE: &str = "aggressive";
const CUSTOM_VALUE: &str = "custom";

const MIN_WINDOWS_10_1809_BUILD: u32 = 17763;

const DEFAULT_INPUT_SERVICE_ENABLED: u32 = 1;
const DEFAULT_INPUT_SERVICE_ENABLED_FOR_CCI: u32 = 1;
const SOFT_DISABLED_VALUE: u32 = 0;
const DEFAULT_SERVICE_DLL_UNLOAD_ON_STOP: u32 = 0;
const AGGRESSIVE_SERVICE_DLL_UNLOAD_ON_STOP: u32 = 1;
const DEFAULT_SERVICE_START: u32 = 3;
const DISABLED_SERVICE_START: u32 = 4;
const AGGRESSIVE_DISABLE_THREAD_INPUT_MANAGER: u32 = 1;

const SERVICE_DOES_NOT_EXIST_HRESULT: i32 = -2147023836;
const SERVICE_NOT_ACTIVE_HRESULT: i32 = -2147023834;

const INPUT_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Input",
};

const CTFMON_SERVICE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\ctfmon",
};

const CTFMON_BACKUP_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Winsentials\TweakBackups\disable_ctf_ctfmon",
};

struct CtfState {
    input_service_enabled: u32,
    input_service_enabled_for_cci: u32,
    service_dll: Option<RawRegValue>,
    service_dll_unload_on_stop: u32,
    service_start: u32,
    disable_thread_input_manager: Option<u32>,
}

enum DwordSnapshot {
    Missing,
    Present(u32),
}

enum RawValueSnapshot {
    Missing,
    Present(RawRegValue),
}

struct CtfSnapshot {
    input_service_enabled: DwordSnapshot,
    input_service_enabled_for_cci: DwordSnapshot,
    service_dll: RawValueSnapshot,
    service_dll_unload_on_stop: DwordSnapshot,
    service_start: DwordSnapshot,
    disable_thread_input_manager: DwordSnapshot,
}

struct ServiceHandle(SC_HANDLE);

impl Drop for ServiceHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseServiceHandle(self.0);
        }
    }
}

pub struct DisableCtfCtfmonTweak {
    meta: TweakMeta,
}

impl Default for DisableCtfCtfmonTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableCtfCtfmonTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_ctf_ctfmon".into(),
                category: "input".into(),
                name: "input.tweaks.disableCtfCtfmon.name".into(),
                short_description: "input.tweaks.disableCtfCtfmon.shortDescription".into(),
                detail_description: "input.tweaks.disableCtfCtfmon.detailDescription".into(),
                control: TweakControlType::Dropdown {
                    options: vec![
                        TweakOption {
                            label: "input.tweaks.disableCtfCtfmon.options.default".into(),
                            value: DEFAULT_VALUE.into(),
                        },
                        TweakOption {
                            label: "input.tweaks.disableCtfCtfmon.options.soft".into(),
                            value: SOFT_VALUE.into(),
                        },
                        TweakOption {
                            label: "input.tweaks.disableCtfCtfmon.options.aggressive".into(),
                            value: AGGRESSIVE_VALUE.into(),
                        },
                    ],
                },
                current_value: DEFAULT_VALUE.into(),
                default_value: DEFAULT_VALUE.into(),
                recommended_value: SOFT_VALUE.into(),
                risk: RiskLevel::High,
                risk_description: Some("input.tweaks.disableCtfCtfmon.riskDescription".into()),
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

    fn read_optional_dword(key: &RegKey, name: &str) -> Result<Option<u32>, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn read_optional_raw_value(key: &RegKey, name: &str) -> Result<Option<RawRegValue>, AppError> {
        match key.get_raw_value(name) {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn read_state(&self) -> Result<CtfState, AppError> {
        Ok(CtfState {
            input_service_enabled: Self::read_dword_or_default(
                &INPUT_KEY,
                "InputServiceEnabled",
                DEFAULT_INPUT_SERVICE_ENABLED,
            )?,
            input_service_enabled_for_cci: Self::read_dword_or_default(
                &INPUT_KEY,
                "InputServiceEnabledForCCI",
                DEFAULT_INPUT_SERVICE_ENABLED_FOR_CCI,
            )?,
            service_dll: Self::read_optional_raw_value(&CTFMON_SERVICE_KEY, "ServiceDll")?,
            service_dll_unload_on_stop: Self::read_dword_or_default(
                &CTFMON_SERVICE_KEY,
                "ServiceDllUnloadOnStop",
                DEFAULT_SERVICE_DLL_UNLOAD_ON_STOP,
            )?,
            service_start: Self::read_dword_or_default(
                &CTFMON_SERVICE_KEY,
                "Start",
                DEFAULT_SERVICE_START,
            )?,
            disable_thread_input_manager: Self::read_optional_dword(
                &INPUT_KEY,
                "Disable Thread Input Manager",
            )?,
        })
    }

    fn is_soft(state: &CtfState) -> bool {
        state.input_service_enabled == SOFT_DISABLED_VALUE
            && state.input_service_enabled_for_cci == SOFT_DISABLED_VALUE
    }

    fn is_aggressive(state: &CtfState) -> bool {
        Self::is_soft(state)
            && state
                .service_dll
                .as_ref()
                .is_some_and(|value| value.vtype == REG_EXPAND_SZ)
            && state.service_dll_unload_on_stop == AGGRESSIVE_SERVICE_DLL_UNLOAD_ON_STOP
            && state.service_start == DISABLED_SERVICE_START
            && state.disable_thread_input_manager == Some(AGGRESSIVE_DISABLE_THREAD_INPUT_MANAGER)
    }

    fn is_default(state: &CtfState) -> bool {
        state.input_service_enabled == DEFAULT_INPUT_SERVICE_ENABLED
            && state.input_service_enabled_for_cci == DEFAULT_INPUT_SERVICE_ENABLED_FOR_CCI
            && state.service_dll_unload_on_stop == DEFAULT_SERVICE_DLL_UNLOAD_ON_STOP
            && state.service_start == DEFAULT_SERVICE_START
            && state.disable_thread_input_manager.is_none()
    }

    fn snapshot_dword(key: &RegKey, name: &str) -> Result<DwordSnapshot, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(DwordSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DwordSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn snapshot_raw_value(key: &RegKey, name: &str) -> Result<RawValueSnapshot, AppError> {
        match key.get_raw_value(name) {
            Ok(value) => Ok(RawValueSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(RawValueSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn capture_snapshot() -> Result<CtfSnapshot, AppError> {
        Ok(CtfSnapshot {
            input_service_enabled: Self::snapshot_dword(&INPUT_KEY, "InputServiceEnabled")?,
            input_service_enabled_for_cci: Self::snapshot_dword(
                &INPUT_KEY,
                "InputServiceEnabledForCCI",
            )?,
            service_dll: Self::snapshot_raw_value(&CTFMON_SERVICE_KEY, "ServiceDll")?,
            service_dll_unload_on_stop: Self::snapshot_dword(
                &CTFMON_SERVICE_KEY,
                "ServiceDllUnloadOnStop",
            )?,
            service_start: Self::snapshot_dword(&CTFMON_SERVICE_KEY, "Start")?,
            disable_thread_input_manager: Self::snapshot_dword(
                &INPUT_KEY,
                "Disable Thread Input Manager",
            )?,
        })
    }

    fn restore_dword(key: &RegKey, name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => key.delete_value(name),
            DwordSnapshot::Present(value) => key.set_dword(name, *value),
        }
    }

    fn restore_raw_value(
        key: &RegKey,
        name: &str,
        snapshot: &RawValueSnapshot,
    ) -> Result<(), AppError> {
        match snapshot {
            RawValueSnapshot::Missing => key.delete_value(name),
            RawValueSnapshot::Present(value) => key.set_raw_value(name, value),
        }
    }

    fn restore_snapshot(snapshot: &CtfSnapshot) -> Result<(), AppError> {
        let mut errors = Vec::new();

        if let Err(error) = Self::restore_dword(
            &INPUT_KEY,
            "InputServiceEnabled",
            &snapshot.input_service_enabled,
        ) {
            errors.push(format!("InputServiceEnabled: {error}"));
        }

        if let Err(error) = Self::restore_dword(
            &INPUT_KEY,
            "InputServiceEnabledForCCI",
            &snapshot.input_service_enabled_for_cci,
        ) {
            errors.push(format!("InputServiceEnabledForCCI: {error}"));
        }

        if let Err(error) =
            Self::restore_raw_value(&CTFMON_SERVICE_KEY, "ServiceDll", &snapshot.service_dll)
        {
            errors.push(format!("ServiceDll: {error}"));
        }

        if let Err(error) = Self::restore_dword(
            &CTFMON_SERVICE_KEY,
            "ServiceDllUnloadOnStop",
            &snapshot.service_dll_unload_on_stop,
        ) {
            errors.push(format!("ServiceDllUnloadOnStop: {error}"));
        }

        if let Err(error) =
            Self::restore_dword(&CTFMON_SERVICE_KEY, "Start", &snapshot.service_start)
        {
            errors.push(format!("Start: {error}"));
        }

        if let Err(error) = Self::restore_dword(
            &INPUT_KEY,
            "Disable Thread Input Manager",
            &snapshot.disable_thread_input_manager,
        ) {
            errors.push(format!("Disable Thread Input Manager: {error}"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::message(format!(
                "rollback failed: {}",
                errors.join("; ")
            )))
        }
    }

    fn apply_with_rollback<F>(writer: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Result<(), AppError>,
    {
        let snapshot = Self::capture_snapshot()?;

        match writer() {
            Ok(()) => Ok(()),
            Err(error) => match Self::restore_snapshot(&snapshot) {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(AppError::message(format!("{error}; {rollback_error}"))),
            },
        }
    }

    fn write_soft_values() -> Result<(), AppError> {
        INPUT_KEY.set_dword("InputServiceEnabled", SOFT_DISABLED_VALUE)?;
        INPUT_KEY.set_dword("InputServiceEnabledForCCI", SOFT_DISABLED_VALUE)
    }

    fn write_aggressive_values() -> Result<(), AppError> {
        Self::persist_aggressive_originals()?;
        Self::write_soft_values()?;
        CTFMON_SERVICE_KEY.set_raw_value(
            "ServiceDll",
            &expand_string_value(r"%SystemRoot%\system32\ctfmon.dll"),
        )?;
        CTFMON_SERVICE_KEY.set_dword(
            "ServiceDllUnloadOnStop",
            AGGRESSIVE_SERVICE_DLL_UNLOAD_ON_STOP,
        )?;
        CTFMON_SERVICE_KEY.set_dword("Start", DISABLED_SERVICE_START)?;
        INPUT_KEY.set_dword(
            "Disable Thread Input Manager",
            AGGRESSIVE_DISABLE_THREAD_INPUT_MANAGER,
        )?;
        Self::stop_service("ctfmon")
    }

    fn write_default_values() -> Result<(), AppError> {
        INPUT_KEY.set_dword("InputServiceEnabled", DEFAULT_INPUT_SERVICE_ENABLED)?;
        INPUT_KEY.set_dword(
            "InputServiceEnabledForCCI",
            DEFAULT_INPUT_SERVICE_ENABLED_FOR_CCI,
        )?;
        Self::restore_aggressive_originals()
    }

    fn persist_aggressive_originals() -> Result<(), AppError> {
        if !CTFMON_BACKUP_KEY.key_exists()? {
            let service_dll = Self::snapshot_raw_value(&CTFMON_SERVICE_KEY, "ServiceDll")?;
            let service_dll_unload_on_stop =
                Self::snapshot_dword(&CTFMON_SERVICE_KEY, "ServiceDllUnloadOnStop")?;
            let service_start = Self::snapshot_dword(&CTFMON_SERVICE_KEY, "Start")?;
            let disable_thread_input_manager =
                Self::snapshot_dword(&INPUT_KEY, "Disable Thread Input Manager")?;

            Self::write_backup_raw_value("ServiceDll", &service_dll)?;
            Self::write_backup_dword("ServiceDllUnloadOnStop", &service_dll_unload_on_stop)?;
            Self::write_backup_dword("Start", &service_start)?;
            Self::write_backup_dword(
                "Disable Thread Input Manager",
                &disable_thread_input_manager,
            )?;
        }

        Ok(())
    }

    fn restore_aggressive_originals() -> Result<(), AppError> {
        let service_dll = Self::read_backup_raw_value("ServiceDll")?.unwrap_or_else(|| {
            RawValueSnapshot::Present(expand_string_value(r"%SystemRoot%\system32\ctfmon.dll"))
        });
        let service_dll_unload_on_stop = Self::read_backup_dword("ServiceDllUnloadOnStop")?
            .unwrap_or(DwordSnapshot::Present(DEFAULT_SERVICE_DLL_UNLOAD_ON_STOP));
        let service_start = Self::read_backup_dword("Start")?
            .unwrap_or(DwordSnapshot::Present(DEFAULT_SERVICE_START));
        let disable_thread_input_manager = Self::read_backup_dword("Disable Thread Input Manager")?
            .unwrap_or(DwordSnapshot::Missing);

        Self::restore_raw_value(&CTFMON_SERVICE_KEY, "ServiceDll", &service_dll)?;
        Self::restore_dword(
            &CTFMON_SERVICE_KEY,
            "ServiceDllUnloadOnStop",
            &service_dll_unload_on_stop,
        )?;
        Self::restore_dword(&CTFMON_SERVICE_KEY, "Start", &service_start)?;
        Self::restore_dword(
            &INPUT_KEY,
            "Disable Thread Input Manager",
            &disable_thread_input_manager,
        )
    }

    fn write_backup_dword(name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => CTFMON_BACKUP_KEY.set_dword(&format!("{name}__Missing"), 1),
            DwordSnapshot::Present(value) => {
                CTFMON_BACKUP_KEY.set_dword(&format!("{name}__Missing"), 0)?;
                CTFMON_BACKUP_KEY.set_dword(name, *value)
            }
        }
    }

    fn read_backup_dword(name: &str) -> Result<Option<DwordSnapshot>, AppError> {
        let missing_name = format!("{name}__Missing");

        match CTFMON_BACKUP_KEY.get_dword(&missing_name) {
            Ok(1) => Ok(Some(DwordSnapshot::Missing)),
            Ok(_) => Ok(Some(DwordSnapshot::Present(
                CTFMON_BACKUP_KEY.get_dword(name)?,
            ))),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn write_backup_raw_value(name: &str, snapshot: &RawValueSnapshot) -> Result<(), AppError> {
        match snapshot {
            RawValueSnapshot::Missing => {
                CTFMON_BACKUP_KEY.set_dword(&format!("{name}__Missing"), 1)
            }
            RawValueSnapshot::Present(value) => {
                CTFMON_BACKUP_KEY.set_dword(&format!("{name}__Missing"), 0)?;
                CTFMON_BACKUP_KEY.set_raw_value(name, value)
            }
        }
    }

    fn read_backup_raw_value(name: &str) -> Result<Option<RawValueSnapshot>, AppError> {
        let missing_name = format!("{name}__Missing");

        match CTFMON_BACKUP_KEY.get_dword(&missing_name) {
            Ok(1) => Ok(Some(RawValueSnapshot::Missing)),
            Ok(_) => Ok(Some(RawValueSnapshot::Present(
                CTFMON_BACKUP_KEY.get_raw_value(name)?,
            ))),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn stop_service(service_name: &str) -> Result<(), AppError> {
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
}

impl Tweak for DisableCtfCtfmonTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            SOFT_VALUE => Self::apply_with_rollback(Self::write_soft_values),
            AGGRESSIVE_VALUE => Self::apply_with_rollback(Self::write_aggressive_values),
            DEFAULT_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::apply_with_rollback(Self::write_default_values)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let state = self.read_state()?;
        let is_default = Self::is_default(&state);

        Ok(TweakStatus {
            current_value: if Self::is_aggressive(&state) {
                AGGRESSIVE_VALUE.into()
            } else if Self::is_soft(&state) {
                SOFT_VALUE.into()
            } else if is_default {
                DEFAULT_VALUE.into()
            } else {
                CUSTOM_VALUE.into()
            },
            is_default,
        })
    }
}

fn expand_string_value(value: &str) -> RawRegValue {
    RawRegValue {
        bytes: value
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
        vtype: REG_EXPAND_SZ,
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
