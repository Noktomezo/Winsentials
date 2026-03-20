#[cfg(target_os = "windows")]
pub fn get_perf_info() -> (u32, u32, u32) {
    use windows::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};
    let mut info: PERFORMANCE_INFORMATION = unsafe { std::mem::zeroed() };
    info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
    if unsafe { K32GetPerformanceInfo(&mut info, info.cb).as_bool() } {
        (info.ProcessCount, info.ThreadCount, info.HandleCount)
    } else {
        (0, 0, 0)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_perf_info() -> (u32, u32, u32) {
    (0, 0, 0)
}

#[cfg(target_os = "windows")]
pub fn pdh_open_cpu_perf_query() -> Option<(isize, isize)> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let mut query = PDH_HQUERY(std::ptr::null_mut());
        if PdhOpenQueryW(windows::core::PCWSTR(std::ptr::null()), 0, &mut query) != 0 {
            return None;
        }
        let path = windows::core::w!(r"\Processor Information(_Total)\% Processor Performance");
        let mut counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, path, 0, &mut counter) != 0 {
            let _ = PdhCloseQuery(query);
            return None;
        }
        let _ = PdhCollectQueryData(query);
        Some((query.0 as isize, counter.0 as isize))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pdh_open_cpu_perf_query() -> Option<(isize, isize)> {
    None
}

#[cfg(target_os = "windows")]
pub fn pdh_collect_cpu_perf_pct(query_raw: isize, counter_raw: isize) -> Option<f64> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let query = PDH_HQUERY(query_raw as *mut _);
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
        let _ = PdhCollectQueryData(query);
        let mut val: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
        let r = PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut val);
        if r != 0 {
            return None;
        }
        Some(val.Anonymous.doubleValue)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pdh_collect_cpu_perf_pct(_query_raw: isize, _counter_raw: isize) -> Option<f64> {
    None
}
