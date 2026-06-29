use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::core::Error as WindowsError;

pub struct ComGuard {
    owned: bool,
}

impl ComGuard {
    pub fn new() -> windows::core::Result<Self> {
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status.is_ok() {
            return Ok(Self { owned: true });
        }

        let error = WindowsError::from(status);
        let code = error.code();

        // S_FALSE: already initialized on this thread with the same apartment.
        // Reference count was incremented, so we must call CoUninitialize on drop.
        if code == S_FALSE {
            return Ok(Self { owned: true });
        }

        // RPC_E_CHANGED_MODE: already initialized with a different apartment.
        // We didn't increment the count, so don't call CoUninitialize.
        if code == RPC_E_CHANGED_MODE {
            return Ok(Self { owned: false });
        }

        Err(error)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
