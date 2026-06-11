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
    <section className="flex h-fit flex-col overflow-hidden rounded-lg border border-border/70 bg-card">
      <button
        className="group/summary flex min-w-0 flex-1 cursor-pointer items-center justify-between gap-3 p-4 text-left focus-visible:outline-none hover:bg-accent/10 transition-colors"
        onClick={handleNavigate}
        onFocus={handlePointerIntent}
        onMouseEnter={handlePointerIntent}
        type="button"
      >
        <div className="flex min-w-0 flex-1 items-center gap-3">
          <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md transition-colors group-hover/summary:bg-accent/40">
            <Icon className="size-4" />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-sm font-medium text-foreground transition-colors group-hover/summary:text-primary">
              {title}
            </h2>
            <p className="mt-1 text-xs text-muted-foreground line-clamp-1">
              {description}
            </p>
          </div>
        </div>
        <ChevronRight className="size-4 shrink-0 text-muted-foreground transition-transform group-hover/summary:translate-x-0.5" />
      </button>
    </section>
  )
}

function ToolsPage() {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="tweak-card-grid">
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
