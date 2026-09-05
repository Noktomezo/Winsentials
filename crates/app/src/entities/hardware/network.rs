use std::collections::HashMap;
use std::time::Instant;

use windows_sys::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetIfTable2, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211, MIB_IF_TABLE2,
};
use windows_sys::Win32::NetworkManagement::Ndis::{
    MediaConnectStateConnected, NET_IF_OPER_STATUS_UP,
};

use super::types::{CPU_HISTORY_SAMPLES, NetworkInfo, NetworkKind, throughput_scale};

#[derive(Clone, Debug)]
pub(crate) struct NetworkCounters {
    pub(crate) id: u32,
    pub(crate) interface_name: String,
    pub(crate) adapter_name: String,
    pub(crate) kind: NetworkKind,
    pub(crate) link_speed_mbps: u64,
    pub(crate) rx_octets: u64,
    pub(crate) tx_octets: u64,
}

fn utf16_z(value: &[u16]) -> String {
    let len = value
        .iter()
        .position(|&code| code == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..len]).trim().to_string()
}

pub(crate) fn is_visible_network_interface(
    interface_type: u32,
    oper_status: i32,
    media_state: i32,
    physical_address_length: u32,
    status_flags: u8,
) -> bool {
    matches!(interface_type, IF_TYPE_ETHERNET_CSMACD | IF_TYPE_IEEE80211)
        && oper_status == NET_IF_OPER_STATUS_UP
        && media_state == MediaConnectStateConnected
        && physical_address_length > 0
        && status_flags & 0b0000_0001 != 0
        && status_flags & 0b0000_0010 == 0
}

pub(crate) fn query_connected_networks() -> Vec<NetworkCounters> {
    let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    #[allow(unsafe_code, clippy::borrow_as_ptr)]
    let status = unsafe { GetIfTable2(&raw mut table_ptr) };
    if status != 0 || table_ptr.is_null() {
        return Vec::new();
    }

    let mut networks = Vec::new();

    #[allow(unsafe_code)]
    unsafe {
        let table = &*table_ptr;
        let rows_slice =
            std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as usize);

        for row in rows_slice {
            let kind = match row.Type {
                IF_TYPE_ETHERNET_CSMACD => NetworkKind::Ethernet,
                IF_TYPE_IEEE80211 => NetworkKind::Wifi,
                _ => continue,
            };
            if is_visible_network_interface(
                row.Type,
                row.OperStatus,
                row.MediaConnectState,
                row.PhysicalAddressLength,
                row.InterfaceAndOperStatusFlags._bitfield,
            ) {
                networks.push(NetworkCounters {
                    id: row.InterfaceIndex,
                    interface_name: utf16_z(&row.Alias),
                    adapter_name: utf16_z(&row.Description),
                    kind,
                    link_speed_mbps: row.ReceiveLinkSpeed.max(row.TransmitLinkSpeed) / 1_000_000,
                    rx_octets: row.InOctets,
                    tx_octets: row.OutOctets,
                });
            }
        }

        FreeMibTable(table_ptr.cast());
    }

    networks.sort_by(|left, right| left.interface_name.cmp(&right.interface_name));
    networks
}

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        let val = bytes_per_sec / (1024.0 * 1024.0 * 1024.0);
        format!("{val:.1} ГБ/с")
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        let val = bytes_per_sec / (1024.0 * 1024.0);
        format!("{val:.1} МБ/с")
    } else if bytes_per_sec >= 1024.0 {
        let val = bytes_per_sec / 1024.0;
        format!("{val:.0} КБ/с")
    } else {
        format!("{bytes_per_sec:.0} Б/с")
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn sample_networks(
    network_snapshots: &mut HashMap<u32, (u64, u64)>,
    network_rx_histories: &mut HashMap<u32, Vec<f32>>,
    network_tx_histories: &mut HashMap<u32, Vec<f32>>,
    last_network_scales: &mut HashMap<u32, f32>,
    elapsed_secs: f64,
    sample_instant: Instant,
) -> Vec<NetworkInfo> {
    let mut networks = Vec::new();
    for current in query_connected_networks() {
        let previous = network_snapshots.insert(current.id, (current.rx_octets, current.tx_octets));
        let (delta_rx, delta_tx) = previous.map_or((0, 0), |(rx, tx)| {
            (
                current.rx_octets.saturating_sub(rx),
                current.tx_octets.saturating_sub(tx),
            )
        });
        let rx_speed_bps = delta_rx as f64 / elapsed_secs;
        let tx_speed_bps = delta_tx as f64 / elapsed_secs;
        let rx_mbps = (rx_speed_bps * 8.0 / 1_000_000.0) as f32;
        let tx_mbps = (tx_speed_bps * 8.0 / 1_000_000.0) as f32;

        let rx_history = network_rx_histories
            .entry(current.id)
            .or_insert_with(|| vec![0.0; CPU_HISTORY_SAMPLES]);
        rx_history.rotate_left(1);
        rx_history[CPU_HISTORY_SAMPLES - 1] = rx_mbps;

        let tx_history = network_tx_histories
            .entry(current.id)
            .or_insert_with(|| vec![0.0; CPU_HISTORY_SAMPLES]);
        tx_history.rotate_left(1);
        tx_history[CPU_HISTORY_SAMPLES - 1] = tx_mbps;

        let target_scale = throughput_scale(rx_history, tx_history);
        let previous_throughput_scale = last_network_scales
            .insert(current.id, target_scale)
            .unwrap_or(target_scale);

        networks.push(NetworkInfo {
            id: current.id,
            interface_name: current.interface_name.into(),
            adapter_name: current.adapter_name.into(),
            kind: current.kind,
            link_speed_mbps: current.link_speed_mbps,
            rx_speed: format_speed(rx_speed_bps).into(),
            tx_speed: format_speed(tx_speed_bps).into(),
            rx_history_15s: rx_history.clone(),
            tx_history_15s: tx_history.clone(),
            previous_throughput_scale,
            throughput_scale: target_scale,
            sample_instant,
        });
    }
    networks
}