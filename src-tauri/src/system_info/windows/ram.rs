#[cfg(target_os = "windows")]
pub fn get_ram_perf() -> (u64, u64, u64, u64, u64, u64) {
    use windows::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};

    let page = unsafe {
        let mut si = std::mem::zeroed::<windows::Win32::System::SystemInformation::SYSTEM_INFO>();
        windows::Win32::System::SystemInformation::GetSystemInfo(&mut si);
        si.dwPageSize as u64
    };

    let (committed, commit_limit, cached, paged, nonpaged) = unsafe {
        let mut info = std::mem::zeroed::<PERFORMANCE_INFORMATION>();
        info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
        if K32GetPerformanceInfo(&mut info, info.cb) != false {
            (
                info.CommitTotal as u64 * page,
                info.CommitLimit as u64 * page,
                info.SystemCache as u64 * page,
                info.KernelPaged as u64 * page,
                info.KernelNonpaged as u64 * page,
            )
        } else {
            (0, 0, 0, 0, 0)
        }
    };

    let compressed = get_compressed_memory_bytes();

    (committed, commit_limit, cached, compressed, paged, nonpaged)
}

#[cfg(not(target_os = "windows"))]
pub fn get_ram_perf() -> (u64, u64, u64, u64, u64, u64) {
    (0, 0, 0, 0, 0, 0)
}

#[cfg(target_os = "windows")]
fn get_compressed_memory_bytes() -> u64 {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    fn is_memory_compression_process(name: &[u16]) -> bool {
        let process_name = String::from_utf16_lossy(name).to_ascii_lowercase();
        matches!(
            process_name.as_str(),
            "memory compression" | "memcompression"
        )
    }

    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return 0,
        };

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snap, &mut entry).is_err() {
            let _ = CloseHandle(snap);
            return 0;
        }

        let mut result: u64 = 0;

        loop {
            let name_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = &entry.szExeFile[..name_len];

            if is_memory_compression_process(name) {
                let handle = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    entry.th32ProcessID,
                )
                .or_else(|_| {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION,
                        false,
                        entry.th32ProcessID,
                    )
                });

                if let Ok(handle) = handle {
                    let mut mem: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                    mem.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                    if K32GetProcessMemoryInfo(
                        handle,
                        &mut mem as *mut PROCESS_MEMORY_COUNTERS_EX
                            as *mut windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS,
                        mem.cb,
                    ) != false
                    {
                        result = mem.WorkingSetSize as u64;
                    }
                    let _ = CloseHandle(handle);
                }
                break;
            }

            if Process32NextW(snap, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snap);
        result
    }
}

#[cfg(not(target_os = "windows"))]
fn get_compressed_memory_bytes() -> u64 {
    0
}
