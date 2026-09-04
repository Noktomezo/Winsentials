const REG_AFD_PARAMS: &str = r"SYSTEM\CurrentControlSet\Services\Afd\Parameters";
const FAST_SEND_DATAGRAM_THRESHOLD: &str = "FastSendDatagramThreshold";
const FAST_COPY_RECEIVE_THRESHOLD: &str = "FastCopyReceiveThreshold";
const FAST_BUFFER_SIZE_64K: u32 = 65536;

#[must_use]
pub fn is_fast_send_copy_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_AFD_PARAMS)
            .is_ok_and(|key| {
                let send_ok = key
                    .get_u32(FAST_SEND_DATAGRAM_THRESHOLD)
                    .is_ok_and(|val| val >= FAST_BUFFER_SIZE_64K);
                let copy_ok = key
                    .get_u32(FAST_COPY_RECEIVE_THRESHOLD)
                    .is_ok_and(|val| val >= FAST_BUFFER_SIZE_64K);
                send_ok && copy_ok
            })
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_fast_send_copy(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_AFD_PARAMS)
            .map_err(|error| format!("Failed to open AFD parameters key: {error}"))?;
        if applied {
            key.set_u32(FAST_SEND_DATAGRAM_THRESHOLD, FAST_BUFFER_SIZE_64K)
                .map_err(|error| format!("Failed to set FastSendDatagramThreshold: {error}"))?;
            key.set_u32(FAST_COPY_RECEIVE_THRESHOLD, FAST_BUFFER_SIZE_64K)
                .map_err(|error| format!("Failed to set FastCopyReceiveThreshold: {error}"))?;
        } else {
            let _ = key.remove_value(FAST_SEND_DATAGRAM_THRESHOLD);
            let _ = key.remove_value(FAST_COPY_RECEIVE_THRESHOLD);
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_send_copy_check_runs_without_panic() {
        let _ = is_fast_send_copy_applied();
    }
}
