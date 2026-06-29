import type { WinappDbStatus } from '@/entities/cleanup/model/types'
import { open } from '@tauri-apps/plugin-dialog'
import { Download, FolderOpen, Loader2, RotateCcw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getWinappDbStatus, setCustomWinapp2Path, updateWinappDb } from '@/entities/cleanup/api'
import { cleanupScanCache } from '@/entities/cleanup/model/scan-cache'
import { toast } from '@/shared/lib/toast'
import { Button } from '@/shared/ui/button'

function formatDate(timestamp: number, locale: string): string {
  return new Date(timestamp * 1000).toLocaleDateString(locale, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function WinappDbStatusBar() {
  const { t, i18n } = useTranslation()
  const [status, setStatus] = useState<WinappDbStatus | null>(null)
  const [updating, setUpdating] = useState(false)

  useEffect(() => {
    getWinappDbStatus().then(setStatus).catch((error) => {
      console.error(error)
    })
  }, [])

  function handleUpdate() {
    if (updating) return
    setUpdating(true)
    updateWinappDb()
      .then((newStatus) => {
        setStatus(newStatus)
        cleanupScanCache.refreshAll()
        toast.success(t('cleanup.db.updateSuccess'))
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.db.updateError'))
      })
      .finally(() => {
        setUpdating(false)
      })
  }

  async function handleSetCustomPath() {
    if (updating) return
    setUpdating(true)
    try {
      const selected = await open({
        filters: [{ name: 'Winapp2.ini', extensions: ['ini'] }],
        multiple: false,
      })
      if (typeof selected !== 'string') return
      const newStatus = await setCustomWinapp2Path(selected)
      setStatus(newStatus)
      cleanupScanCache.refreshAll()
      toast.success(t('cleanup.db.customPathSet'))
    }
    catch (error) {
      console.error(error)
      toast.error(t('cleanup.db.customPathError'))
    }
    finally {
      setUpdating(false)
    }
  }

  function handleResetCustomPath() {
    if (updating) return
    setUpdating(true)
    setCustomWinapp2Path(null)
      .then((newStatus) => {
        setStatus(newStatus)
        cleanupScanCache.refreshAll()
        toast.success(t('cleanup.db.customPathReset'))
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.db.customPathError'))
      })
      .finally(() => {
        setUpdating(false)
      })
  }

  const sourceLabel = status
    ? t(`cleanup.db.source${status.source.charAt(0).toUpperCase()}${status.source.slice(1)}`)
    : ''

  const hasCustomPath = status?.source === 'custom'

  return (
    <div className="flex items-center justify-between gap-3 rounded-lg border border-border/70 bg-card p-3">
      <div className="flex flex-col gap-0.5">
        <span className="text-xs font-medium text-foreground">{sourceLabel}</span>
        {status?.lastUpdated && (
          <span className="text-xs text-muted-foreground">
            {t('cleanup.db.lastUpdated', { date: formatDate(status.lastUpdated, i18n.language) })}
          </span>
        )}
      </div>
      <div className="flex items-center gap-2">
        {hasCustomPath
          ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={handleResetCustomPath}
                disabled={updating}
                type="button"
              >
                <RotateCcw className="size-3.5" />
                <span>{t('cleanup.db.resetCustom')}</span>
              </Button>
            )
          : (
              <Button
                variant="ghost"
                size="sm"
                onClick={handleSetCustomPath}
                disabled={updating}
                type="button"
              >
                <FolderOpen className="size-3.5" />
                <span>{t('cleanup.db.setCustom')}</span>
              </Button>
            )}
        <Button
          variant="outline"
          size="sm"
          onClick={handleUpdate}
          disabled={updating}
          type="button"
        >
          {updating
            ? <Loader2 className="size-3.5 animate-spin" />
            : <Download className="size-3.5" />}
          <span>{updating ? t('cleanup.db.updating') : t('cleanup.db.update')}</span>
        </Button>
      </div>
    </div>
  )
}

export default WinappDbStatusBar
