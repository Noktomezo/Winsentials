import type { LucideIcon } from 'lucide-react'
import { useNavigate, useRouter } from '@tanstack/react-router'
import { ArchiveRestore, ChevronRight, Rocket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'

type ToolRoute = '/backup' | '/startup'

interface ToolCardProps {
  description: string
  icon: LucideIcon
  title: string
  to: ToolRoute
}

function ToolCard({
  description,
  icon: Icon,
  title,
  to,
}: ToolCardProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()

  function handleNavigate() {
    void navigate({ to })
  }

  function handlePointerIntent() {
    preloadRouteIntent(() => router.preloadRoute({ to }))
  }

  return (
    <button
      className="group/summary flex min-h-36 cursor-pointer flex-col gap-4 rounded-lg border border-border/70 bg-card p-4 text-left transition-colors hover:bg-accent/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      data-marquee-group="true"
      onClick={handleNavigate}
      onFocus={handlePointerIntent}
      onMouseEnter={handlePointerIntent}
      type="button"
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-accent/60 text-accent-foreground">
            <Icon className="size-4" />
          </span>
          <div className="min-w-0">
            <p className="text-[11px] font-medium uppercase tracking-[0.16em] text-muted-foreground">
              {t('tools.eyebrow')}
            </p>
            <h2 className="text-sm font-medium text-foreground">{title}</h2>
          </div>
        </div>
        <ChevronRight className="size-4 shrink-0 text-muted-foreground transition-transform group-hover/summary:translate-x-0.5" />
      </div>

      <p className="text-sm leading-6 text-muted-foreground">
        {description}
      </p>

      <div className="mt-auto flex items-center justify-between gap-3 border-t border-border/60 pt-4">
        <span className="text-xs font-medium text-foreground">
          {t('tools.open')}
        </span>
        <span className="rounded-md border border-border/60 bg-background/70 px-2 py-1 text-[11px] font-medium text-muted-foreground">
          {title}
        </span>
      </div>
    </button>
  )
}

export function ToolsPage() {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid gap-4 md:grid-cols-2">
        <ToolCard
          description={t('startup.description')}
          icon={Rocket}
          title={t('startup.title')}
          to="/startup"
        />
        <ToolCard
          description={t('backup.description')}
          icon={ArchiveRestore}
          title={t('backup.title')}
          to="/backup"
        />
      </div>
    </section>
  )
}
