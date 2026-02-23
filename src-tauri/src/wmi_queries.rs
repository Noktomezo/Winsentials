#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use serde::Deserialize;
use wmi::WMIConnection;

pub trait HasPnpDeviceId {
  fn pnp_device_id(&self) -> &Option<String>;
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_PhysicalMemory")]
pub struct Win32_PhysicalMemory {
  #[allow(non_snake_case)]
  pub ConfiguredClockSpeed: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_PhysicalMemoryArray")]
pub struct Win32_PhysicalMemoryArray {
  #[allow(non_snake_case)]
  pub MemoryDevices: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_VideoController")]
pub struct Win32_VideoController {
  #[allow(non_snake_case)]
  pub Name: String,
  #[allow(non_snake_case)]
  pub AdapterRAM: Option<u64>,
  #[allow(non_snake_case)]
  pub PNPDeviceID: Option<String>,
}

impl HasPnpDeviceId for Win32_VideoController {
  fn pnp_device_id(&self) -> &Option<String> {
    &self.PNPDeviceID
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_NetworkAdapter")]
pub struct Win32_NetworkAdapter {
  #[allow(non_snake_case, dead_code)]
  pub Name: String,
  #[allow(non_snake_case)]
  pub PNPDeviceID: Option<String>,
}

impl HasPnpDeviceId for Win32_NetworkAdapter {
  fn pnp_device_id(&self) -> &Option<String> {
    &self.PNPDeviceID
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_USBController")]
pub struct Win32_USBController {
  #[allow(non_snake_case, dead_code)]
  pub Name: String,
  #[allow(non_snake_case)]
  pub PNPDeviceID: Option<String>,
}

impl HasPnpDeviceId for Win32_USBController {
  fn pnp_device_id(&self) -> &Option<String> {
    &self.PNPDeviceID
  }
}

pub fn get_wmi_connection() -> Option<WMIConnection> {
  WMIConnection::new().ok()
}
