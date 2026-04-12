export type RiskLevel = 'none' | 'low' | 'medium' | 'high'

export type RequiresAction
  = | { type: 'none' }
    | { type: 'logout' }
    | { type: 'restart_pc' }
    | { type: 'restart_service', serviceName: string }
    | { type: 'restart_app', appName: string }
    | { type: 'restart_device', deviceName: string }

export interface TweakOption {
  label: string
  value: string
}

export interface TweakConflict {
  description: string
}

export type TweakControlType
  = | { kind: 'toggle' }
    | { kind: 'radio', options: TweakOption[] }
    | { kind: 'dropdown', options: TweakOption[] }

export interface TweakMeta {
  id: string
  category: string
  name: string
  shortDescription: string
  detailDescription: string
  control: TweakControlType
  currentValue: string
  defaultValue: string
  recommendedValue: string
  risk: RiskLevel
  riskDescription?: string
  conflicts?: TweakConflict[]
  requiresAction: RequiresAction
  minOsBuild?: number
  minRequiredMemoryGb?: number
  minOsUbr?: number
}

export interface TweakResult {
  success: boolean
  currentValue: string
}

export interface TweakStatus {
  currentValue: string
  isDefault: boolean
}

export interface WindowsVersion {
  build: number
  ubr: number
}
