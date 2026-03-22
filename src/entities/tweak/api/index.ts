import type {
  RequiresAction,
  TweakConflict,
  TweakControlType,
  TweakMeta,
  TweakResult,
  TweakStatus,
  WindowsVersion,
} from '@/entities/tweak/model/types'
import { invoke } from '@tauri-apps/api/core'

interface BackendTweakMeta {
  category: string
  conflicts?: BackendTweakConflict[]
  control: TweakControlType
  current_value: string
  default_value: string
  detail_description: string
  id: string
  min_os_build?: number
  min_os_ubr?: number
  name: string
  recommended_value: string
  requires_action: BackendRequiresAction
  risk: TweakMeta['risk']
  risk_description?: string
  short_description: string
}

interface BackendTweakConflict {
  description: TweakConflict['description']
}

type BackendRequiresAction
  = | { type: 'none' }
    | { type: 'logout' }
    | { type: 'restart_pc' }
    | { type: 'restart_service', service_name: string }
    | { type: 'restart_app', app_name: string }
    | { type: 'restart_device', device_name: string }

interface BackendTweakResult {
  current_value: string
  success: boolean
}

interface BackendTweakStatus {
  current_value: string
  is_default: boolean
}

function mapRequiresAction(action: BackendRequiresAction): RequiresAction {
  switch (action.type) {
    case 'restart_service':
      return { type: action.type, serviceName: action.service_name }
    case 'restart_app':
      return { type: action.type, appName: action.app_name }
    case 'restart_device':
      return { type: action.type, deviceName: action.device_name }
    default:
      return action
  }
}

function mapTweakMeta(meta: BackendTweakMeta): TweakMeta {
  return {
    category: meta.category,
    conflicts: meta.conflicts,
    control: meta.control,
    currentValue: meta.current_value,
    defaultValue: meta.default_value,
    detailDescription: meta.detail_description,
    id: meta.id,
    minOsBuild: meta.min_os_build,
    minOsUbr: meta.min_os_ubr,
    name: meta.name,
    recommendedValue: meta.recommended_value,
    requiresAction: mapRequiresAction(meta.requires_action),
    risk: meta.risk,
    riskDescription: meta.risk_description,
    shortDescription: meta.short_description,
  }
}

function mapTweakResult(result: BackendTweakResult): TweakResult {
  return {
    currentValue: result.current_value,
    success: result.success,
  }
}

function mapTweakStatus(status: BackendTweakStatus): TweakStatus {
  return {
    currentValue: status.current_value,
    isDefault: status.is_default,
  }
}

export async function getTweaksByCategory(category: string): Promise<TweakMeta[]> {
  const tweaks = await invoke<BackendTweakMeta[]>('tweaks_by_category', { category })
  return tweaks.map(mapTweakMeta)
}

export async function applyTweak(id: string, value: string): Promise<TweakResult> {
  const result = await invoke<BackendTweakResult>('tweak_apply', { id, value })
  return mapTweakResult(result)
}

export async function resetTweak(id: string): Promise<TweakResult> {
  const result = await invoke<BackendTweakResult>('tweak_reset', { id })
  return mapTweakResult(result)
}

export async function getTweakStatus(id: string): Promise<TweakStatus> {
  const status = await invoke<BackendTweakStatus>('tweak_status', { id })
  return mapTweakStatus(status)
}

export async function runTweakExtra(id: string) {
  return invoke<void>('tweak_extra', { id })
}

export async function getWindowsBuild() {
  return invoke<WindowsVersion>('get_windows_build')
}

export async function restartPc(): Promise<void> {
  await invoke('restart_pc')
}
