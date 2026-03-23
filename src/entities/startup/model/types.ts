export type StartupSource = 'registry' | 'startup_folder' | 'scheduled_task'
export type StartupScope = 'current_user' | 'all_users'
export type StartupStatus = 'enabled' | 'disabled'

export interface StartupEntry {
  id: string
  name: string
  displayName: string
  source: StartupSource
  scope: StartupScope
  status: StartupStatus
  command: string | null
  targetPath: string | null
  arguments: string | null
  workingDirectory: string | null
  locationLabel: string
  sourceDisplay: string
  runOnce: boolean
  publisher: string | null
  iconDataUrl: string | null
  registryPath: string | null
  taskPath: string | null
  lastError: string | null
}

export interface StartupEntryDetails extends StartupEntry {
  registryHive: string | null
  registryPath: string | null
  registryValueName: string | null
  startupFolderPath: string | null
  startupFilePath: string | null
  taskPath: string | null
  taskAuthor: string | null
  taskDescription: string | null
  taskTriggers: string[]
  taskActions: string[]
  rawXmlPreview: string | null
}

export interface StartupSourceListResponse {
  source: StartupSource
  entries: StartupEntry[]
  error: string | null
}

export type StartupSourceFilter = 'all' | StartupSource
export type StartupStatusFilter = 'all' | StartupStatus
