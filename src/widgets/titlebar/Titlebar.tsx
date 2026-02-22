import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy, Cpu, Maximize2, Minus, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

export function Titlebar() {
  const { t } = useTranslation()
  const [isMaximized, setIsMaximized] = useState(false)

  useEffect(() => {
    let unlisten: (() => void) | undefined

    async function init() {
      const appWindow = getCurrentWindow()
      setIsMaximized(await appWindow.isMaximized())
      const unsubscribe = await appWindow.onResized(async () => {
        setIsMaximized(await appWindow.isMaximized())
      })
      unlisten = unsubscribe
    }

    init()
    return () => unlisten?.()
  }, [])

  async function handleMinimize() {
    await getCurrentWindow().minimize()
  }

  async function handleMaximize() {
    await getCurrentWindow().toggleMaximize()
  }

  async function handleClose() {
    await getCurrentWindow().close()
  }

  async function handleDrag(e: React.MouseEvent) {
    if (e.buttons === 1) {
      if (e.detail === 2) {
        await getCurrentWindow().toggleMaximize()
      }
      else {
        await getCurrentWindow().startDragging()
      }
    }
  }

  const MaximizeIcon = isMaximized ? Copy : Maximize2

  return (
    <div
      className="flex h-8 items-center justify-between border-b border-border bg-card px-3 select-none"
      onMouseDown={handleDrag}
    >
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <Cpu className="h-4 w-4 text-primary" data-tauri-drag-region />
        <span className="text-sm font-medium" data-tauri-drag-region>
          {t('appName')}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onMouseDown={e => e.stopPropagation()}
          onClick={handleMinimize}
          className="group flex h-4 w-4 items-center justify-center rounded-full bg-[#febc2e] cursor-pointer"
        >
          <Minus className="h-3 w-3 text-[#985700] opacity-0 group-hover:opacity-100" strokeWidth={3} />
        </button>
        <button
          type="button"
          onMouseDown={e => e.stopPropagation()}
          onClick={handleMaximize}
          className="group flex h-4 w-4 items-center justify-center rounded-full bg-[#28c840] cursor-pointer"
        >
          <MaximizeIcon className="h-3 w-3 text-[#006500] opacity-0 group-hover:opacity-100" strokeWidth={isMaximized ? 2.5 : 3} />
        </button>
        <button
          type="button"
          onMouseDown={e => e.stopPropagation()}
          onClick={handleClose}
          className="group flex h-4 w-4 items-center justify-center rounded-full bg-[#ff5f57] cursor-pointer"
        >
          <X className="h-3 w-3 text-[#990000] opacity-0 group-hover:opacity-100" strokeWidth={3} />
        </button>
      </div>
    </div>
  )
}
