use std::collections::BTreeSet;

#[cfg(target_os = "windows")]
use std::collections::HashMap;

use crate::system_info::types::NetworkAdapterInfo;

#[cfg(target_os = "windows")]
pub fn gather_network_adapters() -> Vec<NetworkAdapterInfo> {
    use std::ffi::c_void;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::ptr::{addr_of, null_mut};

    use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS, HANDLE};
    use windows::Win32::NetworkManagement::IpHelper::{
        GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses,
        IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::NetworkManagement::WiFi::{
        WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST, WlanCloseHandle,
        WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface,
        wlan_interface_state_connected, wlan_intf_opcode_current_connection,
    };
    use windows::Win32::Networking::WinSock::{
        AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
    };

    const IF_TYPE_ETHERNET_CSMACD: u32 = 6;
    const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;
    const IF_TYPE_IEEE80211: u32 = 71;
    const IF_TYPE_TUNNEL: u32 = 131;
    const IF_OPER_STATUS_UP: i32 = 1;
    const DOT11_PHY_TYPE_HRDSSS_VALUE: u32 = 1;
    const DOT11_PHY_TYPE_OFDM_VALUE: u32 = 2;
    const DOT11_PHY_TYPE_ERP_VALUE: u32 = 4;
    const DOT11_PHY_TYPE_HT_VALUE: u32 = 7;
    const DOT11_PHY_TYPE_VHT_VALUE: u32 = 8;
    const DOT11_PHY_TYPE_HE_VALUE: u32 = 11;

    #[derive(Debug, Clone, Default)]
    struct WifiInterfaceInfo {
        ssid: Option<String>,
        signal_percent: Option<u32>,
        connection_type: Option<String>,
    }

    fn utf16_ptr_to_string(ptr: *const u16) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        let mut len = 0usize;
        unsafe {
            while *ptr.add(len) != 0 {
                len += 1;
            }
            Some(String::from_utf16_lossy(std::slice::from_raw_parts(
                ptr, len,
            )))
        }
    }

    fn utf16_array_to_string(buf: &[u16]) -> Option<String> {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if end == 0 {
            None
        } else {
            Some(String::from_utf16_lossy(&buf[..end]))
        }
    }

    fn dot11_ssid_to_string(ssid: &[u8], len: usize) -> Option<String> {
        if len == 0 || len > ssid.len() {
            return None;
        }
        Some(String::from_utf8_lossy(&ssid[..len]).to_string()).filter(|v| !v.is_empty())
    }

    fn phy_type_to_label(phy_type: u32) -> Option<String> {
        match phy_type {
            DOT11_PHY_TYPE_HE_VALUE => Some("802.11ax".to_string()),
            DOT11_PHY_TYPE_VHT_VALUE => Some("802.11ac".to_string()),
            DOT11_PHY_TYPE_HT_VALUE => Some("802.11n".to_string()),
            DOT11_PHY_TYPE_ERP_VALUE => Some("802.11g".to_string()),
            DOT11_PHY_TYPE_OFDM_VALUE => Some("802.11a".to_string()),
            DOT11_PHY_TYPE_HRDSSS_VALUE => Some("802.11b".to_string()),
            _ => None,
        }
    }

    fn socket_addr_to_string(sockaddr: *const SOCKADDR) -> Option<String> {
        if sockaddr.is_null() {
            return None;
        }
        unsafe {
            match (*sockaddr).sa_family {
                family if family == AF_INET => {
                    let ipv4 = &*(sockaddr as *const SOCKADDR_IN);
                    let octets = ipv4.sin_addr.S_un.S_addr.to_ne_bytes();
                    Some(Ipv4Addr::from(octets).to_string())
                }
                family if family == AF_INET6 => {
                    let ipv6 = &*(sockaddr as *const SOCKADDR_IN6);
                    Some(Ipv6Addr::from(ipv6.sin6_addr.u.Byte).to_string())
                }
                _ => None,
            }
        }
    }

    fn is_routable_ipv4(addr: &str) -> bool {
        !addr.starts_with("169.254.")
    }

    fn is_routable_ipv6(addr: &str) -> bool {
        !addr.to_ascii_lowercase().starts_with("fe80:")
    }

    fn gather_wifi_interfaces() -> HashMap<String, WifiInterfaceInfo> {
        let mut result = HashMap::new();

        unsafe {
            let mut negotiated = 0u32;
            let mut client = HANDLE::default();
            if WlanOpenHandle(2, None, &mut negotiated, &mut client) != ERROR_SUCCESS.0 {
                return result;
            }

            let mut list_ptr: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
            if WlanEnumInterfaces(client, None, &mut list_ptr) == ERROR_SUCCESS.0
                && !list_ptr.is_null()
            {
                let list = &*list_ptr;
                let interfaces = std::slice::from_raw_parts(
                    addr_of!(list.InterfaceInfo) as *const WLAN_INTERFACE_INFO,
                    list.dwNumberOfItems as usize,
                );

                for interface in interfaces {
                    let key = utf16_array_to_string(&interface.strInterfaceDescription)
                        .unwrap_or_default();
                    let mut data_size = 0u32;
                    let mut data_ptr: *mut c_void = null_mut();
                    let mut opcode = Default::default();

                    if WlanQueryInterface(
                        client,
                        &interface.InterfaceGuid,
                        wlan_intf_opcode_current_connection,
                        None,
                        &mut data_size,
                        &mut data_ptr,
                        Some(&mut opcode),
                    ) == ERROR_SUCCESS.0
                        && !data_ptr.is_null()
                    {
                        let attributes = &*(data_ptr as *const WLAN_CONNECTION_ATTRIBUTES);
                        if attributes.isState == wlan_interface_state_connected {
                            let association = &attributes.wlanAssociationAttributes;
                            result.insert(
                                key.clone(),
                                WifiInterfaceInfo {
                                    ssid: dot11_ssid_to_string(
                                        &association.dot11Ssid.ucSSID,
                                        association.dot11Ssid.uSSIDLength as usize,
                                    ),
                                    signal_percent: Some(association.wlanSignalQuality),
                                    connection_type: phy_type_to_label(
                                        association.dot11PhyType.0 as u32,
                                    ),
                                },
                            );
                        }
                        WlanFreeMemory(data_ptr);
                    }
                }

                WlanFreeMemory(list_ptr as *mut c_void);
            }

            let _ = WlanCloseHandle(client, None);
        }

        result
    }

    let wifi_by_description = gather_wifi_interfaces();
    let mut out_len = 15_000u32;

    loop {
        let mut buffer = vec![0u8; out_len as usize];
        let adapter_head = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        let status = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS,
                None,
                Some(adapter_head),
                &mut out_len,
            )
        };

        if status == ERROR_BUFFER_OVERFLOW.0 {
            continue;
        }
        if status != ERROR_SUCCESS.0 {
            return vec![];
        }

        let mut adapters = Vec::new();
        let mut current = adapter_head;

        unsafe {
            while !current.is_null() {
                let adapter = &*current;
                let if_type = adapter.IfType;
                let is_wifi = if_type == IF_TYPE_IEEE80211;
                let is_ethernet = if_type == IF_TYPE_ETHERNET_CSMACD;

                if adapter.OperStatus.0 == IF_OPER_STATUS_UP
                    && (is_wifi || is_ethernet)
                    && if_type != IF_TYPE_SOFTWARE_LOOPBACK
                    && if_type != IF_TYPE_TUNNEL
                {
                    let name = utf16_ptr_to_string(adapter.FriendlyName.0)
                        .or_else(|| utf16_ptr_to_string(adapter.Description.0))
                        .unwrap_or_else(|| "Unknown".to_string());
                    let adapter_description =
                        utf16_ptr_to_string(adapter.Description.0).unwrap_or_else(|| name.clone());
                    let dns_name = utf16_ptr_to_string(adapter.DnsSuffix.0)
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty());

                    if adapter_description
                        .to_ascii_lowercase()
                        .contains("bluetooth")
                        || name.to_ascii_lowercase().contains("bluetooth")
                    {
                        current = adapter.Next;
                        continue;
                    }

                    let mut ipv4_addresses = BTreeSet::new();
                    let mut ipv6_addresses = BTreeSet::new();
                    let mut has_default_gateway = false;

                    let mut unicast = adapter.FirstUnicastAddress;
                    while !unicast.is_null() {
                        if let Some(address) = socket_addr_to_string((*unicast).Address.lpSockaddr)
                        {
                            if address.contains(':') {
                                ipv6_addresses.insert(address);
                            } else {
                                ipv4_addresses.insert(address);
                            }
                        }
                        unicast = (*unicast).Next;
                    }

                    let mut gateway = adapter.FirstGatewayAddress;
                    while !gateway.is_null() {
                        if socket_addr_to_string((*gateway).Address.lpSockaddr).is_some() {
                            has_default_gateway = true;
                            break;
                        }
                        gateway = (*gateway).Next;
                    }

                    let ipv4_addresses: Vec<String> = ipv4_addresses.into_iter().collect();
                    let ipv6_addresses: Vec<String> = ipv6_addresses.into_iter().collect();
                    let has_routable_ipv4 =
                        ipv4_addresses.iter().any(|addr| is_routable_ipv4(addr));
                    let has_routable_ipv6 =
                        ipv6_addresses.iter().any(|addr| is_routable_ipv6(addr));

                    if has_default_gateway || has_routable_ipv4 || has_routable_ipv6 {
                        let wifi_info = wifi_by_description.get(&adapter_description);
                        adapters.push(NetworkAdapterInfo {
                            index: adapters.len(),
                            name,
                            adapter_description: adapter_description.clone(),
                            dns_name,
                            connection_type: if is_wifi {
                                wifi_info
                                    .and_then(|info| info.connection_type.clone())
                                    .unwrap_or_else(|| "Wi-Fi".to_string())
                            } else {
                                "Ethernet".to_string()
                            },
                            ipv4_addresses,
                            ipv6_addresses,
                            is_wifi,
                            ssid: wifi_info.and_then(|info| info.ssid.clone()),
                            signal_percent: wifi_info.and_then(|info| info.signal_percent),
                        });
                    }
                }

                current = adapter.Next;
            }
        }

        return adapters;
    }
}

#[cfg(not(target_os = "windows"))]
pub fn gather_network_adapters() -> Vec<NetworkAdapterInfo> {
    vec![]
}
