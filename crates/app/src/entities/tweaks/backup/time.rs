#![allow(unsafe_code)]

#[must_use]
#[cfg(target_os = "windows")]
pub fn format_local_time_now() -> (String, i64) {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::Globalization::{DATE_SHORTDATE, GetDateFormatEx, GetTimeFormatEx};
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;

    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&raw mut st) };

    let mut date_buf = [0u16; 64];
    let date_len = unsafe {
        GetDateFormatEx(
            std::ptr::null(),
            DATE_SHORTDATE,
            &raw const st,
            std::ptr::null(),
            date_buf.as_mut_ptr(),
            i32::try_from(date_buf.len()).unwrap_or(64),
            std::ptr::null(),
        )
    };

    let mut time_buf = [0u16; 64];
    let time_len = unsafe {
        GetTimeFormatEx(
            std::ptr::null(),
            0,
            &raw const st,
            std::ptr::null(),
            time_buf.as_mut_ptr(),
            i32::try_from(time_buf.len()).unwrap_or(64),
        )
    };

    let formatted = if date_len > 1 && time_len > 1 {
        let date_str = String::from_utf16_lossy(&date_buf[..(date_len as usize - 1)]);
        let time_str = String::from_utf16_lossy(&time_buf[..(time_len as usize - 1)]);
        format!("{date_str}, {time_str}")
    } else {
        format!(
            "{:02}.{:02}.{:04}, {:02}:{:02}:{:02}",
            st.wDay, st.wMonth, st.wYear, st.wHour, st.wMinute, st.wSecond
        )
    };

    let epoch_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));

    (formatted, epoch_secs)
}

#[must_use]
#[cfg(not(target_os = "windows"))]
pub fn format_local_time_now() -> (String, i64) {
    let now = std::time::SystemTime::now();
    let epoch_secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));

    (format!("Epoch {epoch_secs}"), epoch_secs)
}
