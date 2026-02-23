export type AutostartSource = 'Registry' | 'Folder' | 'Task' | 'Service'
export type CriticalLevel = 'None' | 'Warning' | 'Critical'

export interface AutostartItem {
  id: string
  name: string
  publisher: string
  command: string
  location: string
  source: AutostartSource
  is_enabled: boolean
  is_delayed: boolean
  icon_base64: string | null
  critical_level: CriticalLevel
  file_path: string | null
}

export interface FileProperties {
  name: string
  path: string
  size: string
  created: string
  modified: string
  version: string | null
  publisher: string | null
  description: string | null
}
