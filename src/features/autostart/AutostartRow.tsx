import type { AutostartItem } from '@/shared/types/autostart'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { Clock, Copy, FolderOpen, Info, MoreVertical, Trash2 } from 'lucide-react'

import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { openLocation } from '@/shared/api/autostart'
import { cn } from '@/shared/lib/utils'
import { useAutostartStore } from '@/shared/store/autostart'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu'
import { Skeleton } from '@/shared/ui/skeleton'
import { Switch } from '@/shared/ui/switch'
import { FilePropertiesDialog } from './FilePropertiesDialog'

interface AutostartRowProps {
  item: AutostartItem
}

export function AutostartRow({ item }: AutostartRowProps) {
  const { t } = useTranslation()
  const { toggle, delete: deleteItem, enriching } = useAutostartStore()
  const [showProperties, setShowProperties] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [isToggling, setIsToggling] = useState(false)
  const cancelBtnRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (showDeleteConfirm) {
      cancelBtnRef.current?.focus()
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          setShowDeleteConfirm(false)
        }
      }
      window.addEventListener('keydown', handleKeyDown)
      return () => window.removeEventListener('keydown', handleKeyDown)
    }
    return undefined
  }, [showDeleteConfirm])

  const rowClass = cn(
    'flex items-center gap-3 rounded-lg border px-3 py-2 transition-colors',
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
  }

  const handleOpenLocation = async () => {
    console.log('openLocation called with:', item.file_path)
    try {
      if (item.file_path) {
        await openLocation(item.file_path)
      }
    }
    catch (error) {
      console.error('Failed to open location:', error)
    }
  }

  const handleToggle = async (checked: boolean) => {
    setIsToggling(true)
    try {
      await toggle(item.id, checked)
    }
    catch (error) {
      console.error('Failed to toggle:', error)
    }
    finally {
      setIsToggling(false)
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
    }
  }

  return (
    <>
      <div className={rowClass}>
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded bg-muted">
          {enriching && !item.icon_base64
            ? (
                <Skeleton className="h-6 w-6 rounded" />
              )
            : item.icon_base64
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
            {item.start_type && (
              <span className={cn(
                'flex items-center gap-1 rounded px-1.5 py-0.5 text-xs',
                {
                  'bg-purple-500/10 text-purple-600 dark:text-purple-400': item.start_type === 'Boot' || item.start_type === 'System',
                  'bg-green-500/10 text-green-600 dark:text-green-400': item.start_type === 'Auto',
                  'bg-blue-500/10 text-blue-600 dark:text-blue-400': item.start_type === 'Delayed',
                  'bg-amber-500/10 text-amber-600 dark:text-amber-400': item.start_type === 'Manual',
                  'bg-gray-500/10 text-gray-600 dark:text-gray-400': item.start_type === 'Disabled' || item.start_type === 'Unknown',
                },
              )}
              >
                {t(`autostart.startType.${item.start_type}`)}
              </span>
            )}
            {item.is_delayed && !item.start_type && (
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
          <Switch
            checked={item.is_enabled}
            onCheckedChange={handleToggle}
            disabled={isToggling}
            aria-label={t('autostart.toggleAria', { name: item.name })}
          />

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button
                type="button"
                className="rounded p-1 hover:bg-accent cursor-pointer"
                aria-label={t('autostart.openMenu')}
              >
                <MoreVertical className="h-4 w-4" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              {item.file_path && (
                <>
                  <DropdownMenuItem onClick={handleOpenLocation}>
                    <FolderOpen className="mr-2 h-4 w-4" />
                    {t('autostart.openLocation')}
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setShowProperties(true)}>
                    <Info className="mr-2 h-4 w-4" />
                    {t('autostart.properties')}
                  </DropdownMenuItem>
                </>
              )}
              <DropdownMenuItem onClick={handleCopyCommand}>
                <Copy className="mr-2 h-4 w-4" />
                {t('autostart.copyCommand')}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => setShowDeleteConfirm(true)}
                variant="destructive"
              >
                <Trash2 className="mr-2 h-4 w-4" />
                {t('autostart.delete')}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
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
                ref={cancelBtnRef}
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
