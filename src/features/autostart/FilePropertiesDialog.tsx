import type { FileProperties } from '@/shared/types/autostart'
import { X } from 'lucide-react'
import { useEffect, useState } from 'react'

import { useTranslation } from 'react-i18next'
import { getProperties } from '@/shared/api/autostart'

interface FilePropertiesDialogProps {
  path: string
  onClose: () => void
}

export function FilePropertiesDialog({ path, onClose }: FilePropertiesDialogProps) {
  const { t } = useTranslation()
  const [props, setProps] = useState<FileProperties | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    getProperties(path)
      .then(setProps)
      .finally(() => setLoading(false))
  }, [path])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-lg border border-border bg-card">
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h3 className="text-lg font-semibold">{t('autostart.properties')}</h3>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 hover:bg-accent"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="p-4">
          {loading
            ? (
                <div className="text-center text-muted-foreground">{t('common.loading')}</div>
              )
            : props
              ? (
                  <div className="space-y-3 text-sm">
                    <PropertyRow label={t('autostart.propertiesDialog.fileName')} value={props.name} />
                    <PropertyRow label={t('autostart.propertiesDialog.path')} value={props.path} mono />
                    <PropertyRow label={t('autostart.propertiesDialog.size')} value={props.size} />
                    <PropertyRow label={t('autostart.propertiesDialog.created')} value={props.created} />
                    <PropertyRow label={t('autostart.propertiesDialog.modified')} value={props.modified} />
                    {props.version && (
                      <PropertyRow label={t('autostart.propertiesDialog.version')} value={props.version} />
                    )}
                    {props.publisher && (
                      <PropertyRow label={t('autostart.propertiesDialog.publisher')} value={props.publisher} />
                    )}
                    {props.description && (
                      <PropertyRow label={t('autostart.propertiesDialog.description')} value={props.description} />
                    )}
                  </div>
                )
              : (
                  <div className="text-center text-muted-foreground">{t('common.error')}</div>
                )}
        </div>

        <div className="flex justify-end border-t border-border px-4 py-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-border px-4 py-2 text-sm hover:bg-accent"
          >
            {t('common.close')}
          </button>
        </div>
      </div>
    </div>
  )
}

function PropertyRow({ label, value, mono }: { label: string, value: string, mono?: boolean }) {
  return (
    <div className="flex items-start gap-4">
      <span className="w-24 shrink-0 text-muted-foreground">{label}</span>
      <span className={mono ? 'font-mono text-xs break-all' : 'break-all'}>{value}</span>
    </div>
  )
}
