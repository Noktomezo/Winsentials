import type { LucideIcon } from 'lucide-react'
import { useNavigate, useRouter } from '@tanstack/react-router'
import { ChevronRight, Rocket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'

type ToolRoute = '/startup'

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
          <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
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
    </button>
  )
}

function ToolsPage() {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid gap-4 md:grid-cols-[repeat(auto-fit,minmax(20rem,1fr))]">
        <ToolCard
          description={t('startup.description')}
          icon={Rocket}
          title={t('startup.title')}
          to="/startup"
        />
      </div>
    </section>
  )
}

export default ToolsPage
