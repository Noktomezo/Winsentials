use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::run_powershell;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const ENABLED_FLAG: u32 = 1;
const DISABLED_FLAG: u32 = 0;
const DISABLED_FSE_BEHAVIOR: u32 = 2;

const GAME_DVR_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\GameDVR",
};

const GAME_CONFIG_STORE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"System\GameConfigStore",
};

const GAME_DVR_POLICY_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\GameDVR",
};

struct GameDvrState {
    app_capture_enabled: Option<u32>,
    game_dvr_enabled: Option<u32>,
    fse_behavior: Option<u32>,
    fse_behavior_mode: Option<u32>,
    honor_user_fse: Option<u32>,
    dxgi_honor_fse: Option<u32>,
    efse_feature_flags: Option<u32>,
    allow_game_dvr: Option<u32>,
}

pub struct DisableGameDvrTweak {
    meta: TweakMeta,
}

impl Default for DisableGameDvrTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableGameDvrTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_game_dvr".into(),
                category: "performance".into(),
                name: "performance.tweaks.disableGameDvr.name".into(),
                short_description: "performance.tweaks.disableGameDvr.shortDescription".into(),
                detail_description: "performance.tweaks.disableGameDvr.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("performance.tweaks.disableGameDvr.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn read_dword(key: &RegKey, name: &str) -> Result<Option<u32>, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn read_state(&self) -> Result<GameDvrState, AppError> {
        Ok(GameDvrState {
            app_capture_enabled: Self::read_dword(&GAME_DVR_KEY, "AppCaptureEnabled")?,
            game_dvr_enabled: Self::read_dword(&GAME_CONFIG_STORE_KEY, "GameDVR_Enabled")?,
            fse_behavior: Self::read_dword(&GAME_CONFIG_STORE_KEY, "GameDVR_FSEBehavior")?,
            fse_behavior_mode: Self::read_dword(&GAME_CONFIG_STORE_KEY, "GameDVR_FSEBehaviorMode")?,
            honor_user_fse: Self::read_dword(
                &GAME_CONFIG_STORE_KEY,
                "GameDVR_HonorUserFSEBehaviorMode",
            )?,
            dxgi_honor_fse: Self::read_dword(
                &GAME_CONFIG_STORE_KEY,
                "GameDVR_DXGIHonorFSEWindowsCompatible",
            )?,
            efse_feature_flags: Self::read_dword(
                &GAME_CONFIG_STORE_KEY,
                "GameDVR_EFSEFeatureFlags",
            )?,
            allow_game_dvr: Self::read_dword(&GAME_DVR_POLICY_KEY, "AllowGameDVR")?,
        })
    }

    fn is_enabled(state: &GameDvrState) -> bool {
        state.app_capture_enabled == Some(DISABLED_FLAG)
            && state.game_dvr_enabled == Some(DISABLED_FLAG)
            && state.fse_behavior == Some(DISABLED_FSE_BEHAVIOR)
            && state.fse_behavior_mode == Some(DISABLED_FSE_BEHAVIOR)
            && state.honor_user_fse == Some(DISABLED_FLAG)
            && state.dxgi_honor_fse == Some(DISABLED_FLAG)
            && state.efse_feature_flags == Some(DISABLED_FLAG)
            && state.allow_game_dvr == Some(DISABLED_FLAG)
    }

    fn is_default(state: &GameDvrState) -> bool {
        state.app_capture_enabled == Some(ENABLED_FLAG)
            && state.game_dvr_enabled == Some(ENABLED_FLAG)
            && state.fse_behavior.is_none()
            && state.fse_behavior_mode.is_none()
            && state.honor_user_fse.is_none()
            && state.dxgi_honor_fse.is_none()
            && state.efse_feature_flags.is_none()
            && state.allow_game_dvr != Some(DISABLED_FLAG)
    }
}

impl Tweak for DisableGameDvrTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                GAME_DVR_KEY.set_dword("AppCaptureEnabled", DISABLED_FLAG)?;
                GAME_CONFIG_STORE_KEY.set_dword("GameDVR_Enabled", DISABLED_FLAG)?;
                GAME_CONFIG_STORE_KEY.set_dword("GameDVR_FSEBehavior", DISABLED_FSE_BEHAVIOR)?;
                GAME_CONFIG_STORE_KEY
                    .set_dword("GameDVR_FSEBehaviorMode", DISABLED_FSE_BEHAVIOR)?;
                GAME_CONFIG_STORE_KEY
                    .set_dword("GameDVR_HonorUserFSEBehaviorMode", DISABLED_FLAG)?;
                GAME_CONFIG_STORE_KEY
                    .set_dword("GameDVR_DXGIHonorFSEWindowsCompatible", DISABLED_FLAG)?;
                GAME_CONFIG_STORE_KEY.set_dword("GameDVR_EFSEFeatureFlags", DISABLED_FLAG)?;
                GAME_DVR_POLICY_KEY.set_dword("AllowGameDVR", DISABLED_FLAG)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        GAME_DVR_KEY.set_dword("AppCaptureEnabled", ENABLED_FLAG)?;
        GAME_CONFIG_STORE_KEY.set_dword("GameDVR_Enabled", ENABLED_FLAG)?;
        GAME_CONFIG_STORE_KEY.delete_value("GameDVR_FSEBehavior")?;
        GAME_CONFIG_STORE_KEY.delete_value("GameDVR_FSEBehaviorMode")?;
        GAME_CONFIG_STORE_KEY.delete_value("GameDVR_HonorUserFSEBehaviorMode")?;
        GAME_CONFIG_STORE_KEY.delete_value("GameDVR_DXGIHonorFSEWindowsCompatible")?;
        GAME_CONFIG_STORE_KEY.delete_value("GameDVR_EFSEFeatureFlags")?;
        GAME_DVR_POLICY_KEY.delete_value("AllowGameDVR")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let state = self.read_state()?;
        let enabled = Self::is_enabled(&state);
        let is_default = Self::is_default(&state);

        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                "custom".into()
            },
            is_default,
        })
    }

    fn extra(&self) -> Result<(), AppError> {
        run_powershell(
            "$package = Get-AppxPackage Microsoft.XboxGamingOverlay -ErrorAction SilentlyContinue; \
if ($null -eq $package) { Write-Output 'Xbox Game Bar is not installed.'; exit 0 }; \
$package | Remove-AppxPackage -ErrorAction Stop",
        )?;

        Ok(())
    }
}
