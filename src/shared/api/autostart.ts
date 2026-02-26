import type { AutostartItem, EnrichmentData, EnrichRequest, FileProperties } from '@/shared/types/autostart'
import { invoke } from '@tauri-apps/api/core'

export async function getAutostartItems(): Promise<AutostartItem[]> {
  return invoke('get_autostart_items')
}

export async function getAutostartItemsFast(): Promise<AutostartItem[]> {
  return invoke('get_autostart_items_fast')
}

export async function enrichAutostartItems(requests: EnrichRequest[]): Promise<EnrichmentData[]> {
  return invoke('enrich_autostart', { requests })
}

export async function toggleAutostart(id: string, enable: boolean): Promise<void> {
  return invoke('toggle_autostart', { id, enable })
}

export async function deleteAutostart(id: string): Promise<void> {
  return invoke('delete_autostart', { id })
}

export async function openLocation(path: string): Promise<void> {
  return invoke('open_location', { path })
}

export async function getProperties(path: string): Promise<FileProperties> {
  return invoke('get_properties', { path })
}
