import type { AppWebviewMaterial, ResolvedTheme } from '@/shared/config/app'
import { invoke } from '@tauri-apps/api/core'

export async function syncWebviewMaterial(options: {
  material: AppWebviewMaterial
  theme: ResolvedTheme
}) {
  return invoke<boolean>('set_webview_material', options)
}
