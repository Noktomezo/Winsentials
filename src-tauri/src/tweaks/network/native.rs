use std::collections::HashMap;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpCongestionProvider {
    Default,
    NewReno,
    Ctcp,
    Dctcp,
    Ledbat,
    Cubic,
    Bbr2,
    Unknown(u32),
}

impl TcpCongestionProvider {
    pub fn from_code(code: u32) -> Self {
        match code {
            0 => Self::Default,
            1 => Self::NewReno,
            2 => Self::Ctcp,
            3 => Self::Dctcp,
            4 => Self::Ledbat,
            5 => Self::Cubic,
            6 => Self::Bbr2,
            other => Self::Unknown(other),
        }
    }
}

#[cfg(target_os = "windows")]
fn wmi_u32(row: &mut HashMap<String, wmi::Variant>, key: &str) -> Option<u32> {
    match row.remove(key) {
        Some(wmi::Variant::UI1(value)) => Some(value as u32),
        Some(wmi::Variant::UI2(value)) => Some(value as u32),
        Some(wmi::Variant::UI4(value)) => Some(value),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
pub fn tcp_congestion_provider(
    setting_name: &str,
) -> Result<Option<TcpCongestionProvider>, AppError> {
    use wmi::WMIConnection;

    let wmi = WMIConnection::with_namespace_path("ROOT\\StandardCimv2")
        .map_err(|error| AppError::message(error.to_string()))?;

    let query = format!(
        "SELECT SettingName, CongestionProvider FROM MSFT_NetTCPSetting WHERE SettingName='{}'",
        setting_name.replace('"', "")
    );

    let rows: Vec<HashMap<String, wmi::Variant>> = wmi
        .raw_query(&query)
        .map_err(|error| AppError::message(error.to_string()))?;

    Ok(rows
        .into_iter()
        .next()
        .and_then(|mut row| wmi_u32(&mut row, "CongestionProvider"))
        .map(TcpCongestionProvider::from_code))
}

#[cfg(not(target_os = "windows"))]
pub fn tcp_congestion_provider(
    _setting_name: &str,
) -> Result<Option<TcpCongestionProvider>, AppError> {
    Ok(None)
}
