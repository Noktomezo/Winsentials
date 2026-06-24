import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'

interface LiveErrorStateProps {
  message: string
  onRetry: () => void
}

export function LiveErrorState({ message, onRetry }: LiveErrorStateProps) {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
        <p className="text-sm text-muted-foreground">{message}</p>
        <div>
          <Button onClick={onRetry} size="sm" type="button" variant="outline">
            {t('tweaks.actions.retry')}
          </Button>
        </div>
      </section>
    </section>
  )
}
