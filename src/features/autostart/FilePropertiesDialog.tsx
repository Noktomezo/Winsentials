import type { FileProperties } from '@/shared/types/autostart'
import { useEffect, useState } from 'react'

import { useTranslation } from 'react-i18next'
import { getProperties } from '@/shared/api/autostart'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/dialog'

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
    <Dialog open={true} onOpenChange={open => !open && onClose()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{t('autostart.properties')}</DialogTitle>
        </DialogHeader>

        <div className="py-2">
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

        <DialogFooter>
          <button
            type="button"
            onClick={onClose}
            className="cursor-pointer rounded-lg border border-border px-4 py-2 text-sm hover:bg-accent"
          >
            {t('common.close')}
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
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
