import type { AutostartItem } from '@/shared/types/autostart'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { Clock, Copy, FolderOpen, Info, MoreVertical, Power, PowerOff, Trash2 } from 'lucide-react'

import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { useAutostartStore } from '@/shared/store/autostart'
import { FilePropertiesDialog } from './FilePropertiesDialog'

interface AutostartRowProps {
  item: AutostartItem
}

export function AutostartRow({ item }: AutostartRowProps) {
  const { t } = useTranslation()
  const { toggle, delete: deleteItem } = useAutostartStore()
  const [showMenu, setShowMenu] = useState(false)
  const [showProperties, setShowProperties] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)

  const rowClass = cn(
    'flex items-center gap-3 rounded-lg border px-3 py-2 transition-colors hover:bg-accent/50',
    item.critical_level === 'Critical' && 'bg-red-500/5 border-l-2 border-l-red-500',
    item.critical_level === 'Warning' && 'bg-orange-500/5 border-l-2 border-l-orange-500',
  )

  const handleCopyCommand = async () => {
    try {
      await writeText(item.command)
    }
    catch (error) {
      console.error('Failed to copy:', error)
    }
    finally {
      setShowMenu(false)
    }
  }

  const handleOpenLocation = async () => {
    try {
      if (item.file_path) {
        const { openLocation } = await import('@/shared/api/autostart')
        await openLocation(item.file_path)
      }
    }
    catch (error) {
      console.error('Failed to open location:', error)
    }
    finally {
      setShowMenu(false)
    }
  }

  const handleToggle = async () => {
    try {
      await toggle(item.id, !item.is_enabled)
    }
    catch (error) {
      console.error('Failed to toggle:', error)
    }
    finally {
      setShowMenu(false)
    }
  }

  const handleDelete = async () => {
    try {
      await deleteItem(item.id)
    }
    catch (error) {
      console.error('Failed to delete:', error)
    }
    finally {
      setShowDeleteConfirm(false)
      setShowMenu(false)
    }
  }

  return (
    <>
      <div className={rowClass}>
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded bg-muted">
          {item.icon_base64
            ? (
                <img src={item.icon_base64} alt="" className="h-6 w-6 object-contain" />
              )
            : (
                <div className="h-6 w-6 rounded bg-muted-foreground/20" />
              )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium truncate">{item.name}</span>
            {item.is_delayed && (
              <span className="flex items-center gap-1 rounded bg-blue-500/10 px-1.5 py-0.5 text-xs text-blue-600 dark:text-blue-400">
                <Clock className="h-3 w-3" />
                {t('autostart.delayed')}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className="truncate">{item.publisher || t('autostart.unknownPublisher')}</span>
            <span className="text-muted-foreground/50">•</span>
            <span className="truncate">{item.location}</span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <span
            className={cn(
              'rounded-full px-2 py-0.5 text-xs',
              item.is_enabled
                ? 'bg-green-500/10 text-green-600 dark:text-green-400'
                : 'bg-muted text-muted-foreground',
            )}
          >
            {item.is_enabled ? t('autostart.enabled') : t('autostart.disabled')}
          </span>

          <div className="relative">
            <button
              type="button"
              onClick={() => setShowMenu(!showMenu)}
              className="rounded p-1 hover:bg-accent"
              aria-label={showMenu ? t('autostart.closeMenu') : t('autostart.openMenu')}
            >
              <MoreVertical className="h-4 w-4" />
            </button>

            {showMenu && (
              <>
                <div
                  className="fixed inset-0 z-10"
                  onClick={() => setShowMenu(false)}
                />
                <div className="absolute right-0 top-full z-20 mt-1 w-48 rounded-lg border border-border bg-card py-1 shadow-lg">
                  <button
                    type="button"
                    onClick={handleToggle}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-sm hover:bg-accent"
                  >
                    {item.is_enabled
                      ? (
                          <>
                            <PowerOff className="h-4 w-4" />
                            {t('autostart.disable')}
                          </>
                        )
                      : (
                          <>
                            <Power className="h-4 w-4" />
                            {t('autostart.enable')}
                          </>
                        )}
                  </button>

                  {item.file_path && (
                    <>
                      <button
                        type="button"
                        onClick={handleOpenLocation}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-sm hover:bg-accent"
                      >
                        <FolderOpen className="h-4 w-4" />
                        {t('autostart.openLocation')}
                      </button>

                      <button
                        type="button"
                        onClick={() => {
                          setShowProperties(true)
                          setShowMenu(false)
                        }}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-sm hover:bg-accent"
                      >
                        <Info className="h-4 w-4" />
                        {t('autostart.properties')}
                      </button>
                    </>
                  )}

                  <button
                    type="button"
                    onClick={handleCopyCommand}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-sm hover:bg-accent"
                  >
                    <Copy className="h-4 w-4" />
                    {t('autostart.copyCommand')}
                  </button>

                  <div className="my-1 border-t border-border" />

                  <button
                    type="button"
                    onClick={() => setShowDeleteConfirm(true)}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-red-600 hover:bg-red-500/10"
                  >
                    <Trash2 className="h-4 w-4" />
                    {t('autostart.delete')}
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      {showProperties && item.file_path && (
        <FilePropertiesDialog
          path={item.file_path}
          onClose={() => setShowProperties(false)}
        />
      )}

      {showDeleteConfirm && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          role="dialog"
          aria-modal="true"
          aria-labelledby={`deleteDialogTitle-${item.id}`}
        >
          <div className="w-full max-w-md rounded-lg border border-border bg-card p-4">
            <h3 id={`deleteDialogTitle-${item.id}`} className="text-lg font-semibold">
              {item.critical_level === 'Critical'
                ? t('autostart.confirmDeleteCritical', { name: item.name })
                : t('autostart.confirmDelete', { name: item.name })}
            </h3>
            {item.critical_level === 'Warning' && (
              <p className="mt-2 text-sm text-orange-600">
                {t('autostart.confirmDeleteWarning')}
              </p>
            )}
            {item.critical_level === 'Critical' && (
              <p className="mt-2 text-sm text-red-600">
                {t('autostart.criticalWarning')}
              </p>
            )}
            <div className="mt-4 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setShowDeleteConfirm(false)}
                className="rounded-lg border border-border px-4 py-2 text-sm hover:bg-accent"
              >
                {t('common.cancel')}
              </button>
              <button
                type="button"
                onClick={handleDelete}
                className="rounded-lg bg-red-600 px-4 py-2 text-sm text-white hover:bg-red-700"
              >
                {t('autostart.delete')}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
