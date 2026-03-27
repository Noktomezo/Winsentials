import type {
  StartupEntry,
  StartupEntryDetails,
  StartupSource,
  StartupSourceListResponse,
} from '@/entities/startup/model/types'
import { invoke } from '@tauri-apps/api/core'

interface BackendStartupEntry {
  id: string
  name: string
  display_name: string
  source: StartupSource
  scope: StartupEntry['scope']
  status: StartupEntry['status']
  command: string | null
  target_path: string | null
  arguments: string | null
  working_directory: string | null
  location_label: string
  source_display: string
  run_once: boolean
  publisher: string | null
  icon_data_url: string | null
  registry_path: string | null
  task_path: string | null
  last_error: string | null
}

interface BackendStartupEntryDetails extends BackendStartupEntry {
  registry_hive: string | null
  registry_path: string | null
  registry_value_name: string | null
  startup_folder_path: string | null
  startup_file_path: string | null
  task_path: string | null
  task_author: string | null
  task_description: string | null
  task_triggers: string[]
  task_actions: string[]
  raw_xml_preview: string | null
}

interface BackendStartupSourceListResponse {
  source: StartupSource
  entries: BackendStartupEntry[]
  error: string | null
}

function mapEntry(entry: BackendStartupEntry): StartupEntry {
  return {
    id: entry.id,
    name: entry.name,
    displayName: entry.display_name,
    source: entry.source,
    scope: entry.scope,
    status: entry.status,
    command: entry.command,
    targetPath: entry.target_path,
    arguments: entry.arguments,
    workingDirectory: entry.working_directory,
    locationLabel: entry.location_label,
    sourceDisplay: entry.source_display,
    runOnce: entry.run_once,
    publisher: entry.publisher,
    iconDataUrl: entry.icon_data_url,
    registryPath: entry.registry_path,
    taskPath: entry.task_path,
    lastError: entry.last_error,
  }
}

function mapSourceResponse(response: BackendStartupSourceListResponse): StartupSourceListResponse {
  return {
    source: response.source,
    entries: response.entries.map(mapEntry),
    error: response.error,
  }
}

function mapDetails(entry: BackendStartupEntryDetails): StartupEntryDetails {
  return {
    ...mapEntry(entry),
    registryHive: entry.registry_hive,
    registryPath: entry.registry_path,
    registryValueName: entry.registry_value_name,
    startupFolderPath: entry.startup_folder_path,
    startupFilePath: entry.startup_file_path,
    taskPath: entry.task_path,
    taskAuthor: entry.task_author,
    taskDescription: entry.task_description,
    taskTriggers: entry.task_triggers,
    taskActions: entry.task_actions,
    rawXmlPreview: entry.raw_xml_preview,
  }
}

export async function getRegistryStartupEntries(): Promise<StartupSourceListResponse> {
  const response = await invoke<BackendStartupSourceListResponse>('startup_list_registry')
  return mapSourceResponse(response)
}

export async function getStartupFolderEntries(): Promise<StartupSourceListResponse> {
  const response = await invoke<BackendStartupSourceListResponse>('startup_list_startup_folder')
  return mapSourceResponse(response)
}

export async function getScheduledTaskStartupEntries(): Promise<StartupSourceListResponse> {
  const response = await invoke<BackendStartupSourceListResponse>('startup_list_scheduled_tasks')
  return mapSourceResponse(response)
}

export async function hydrateStartupEntries(ids: string[]): Promise<StartupEntry[]> {
  if (ids.length === 0) {
    return []
  }

  const response = await invoke<BackendStartupEntry[]>('startup_hydrate_entries', { ids })
  return response.map(mapEntry)
}

export async function getStartupEntryDetails(id: string): Promise<StartupEntryDetails> {
  const response = await invoke<BackendStartupEntryDetails>('startup_details', { id })
  return mapDetails(response)
}

export async function enableStartupEntry(id: string): Promise<StartupEntry> {
  const response = await invoke<BackendStartupEntry>('startup_enable', { id })
  return mapEntry(response)
}

export async function disableStartupEntry(id: string): Promise<StartupEntry> {
  const response = await invoke<BackendStartupEntry>('startup_disable', { id })
  return mapEntry(response)
}

export async function deleteStartupEntry(id: string): Promise<void> {
  await invoke('startup_delete', { id })
}
