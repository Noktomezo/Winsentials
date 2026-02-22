import type { TweakInfo, TweakState } from '../types/tweak'
import { invoke } from '@tauri-apps/api/core'

export async function getTweaksByCategory(category: string): Promise<TweakInfo[]> {
  return invoke<TweakInfo[]>('get_tweaks_by_category', { category })
}

export async function getTweakInfo(id: string): Promise<TweakInfo | null> {
  return invoke<TweakInfo | null>('get_tweak_info', { id })
}

export async function applyTweak(id: string, value?: string): Promise<TweakState> {
  return invoke<TweakState>('apply_tweak', { id, value })
}

export async function revertTweak(id: string): Promise<TweakState> {
  return invoke<TweakState>('revert_tweak', { id })
}

export async function checkTweak(id: string): Promise<TweakState> {
  return invoke<TweakState>('check_tweak', { id })
}

export async function getAllTweaksInfo(): Promise<TweakInfo[]> {
  return invoke<TweakInfo[]>('get_all_tweaks_info')
}
