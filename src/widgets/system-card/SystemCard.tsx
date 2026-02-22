import type { LucideIcon } from 'lucide-react'
import { Progress } from '@/components/ui/progress'

interface SystemCardProps {
  icon: LucideIcon
  title: string
  value: string
  usage?: number
  metrics?: { label: string, value: string }[]
  progressBars?: { label: string, value: number }[]
}

export function SystemCard({ icon: Icon, title, value, usage, metrics, progressBars }: SystemCardProps) {
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
            <Icon className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h3 className="text-sm font-medium text-muted-foreground">{title}</h3>
            <p className="text-base font-semibold">{value}</p>
          </div>
        </div>
        {usage !== undefined && (
          <span className="text-sm font-medium">
            {usage.toFixed(1)}
            %
          </span>
        )}
      </div>
      {usage !== undefined && (
        <Progress value={usage} className="mt-3" />
      )}
      {metrics && metrics.length > 0 && (
        <div className="mt-3 space-y-1 text-sm">
          {metrics.map((metric, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="text-muted-foreground">
                {metric.label}
                :
              </span>
              <span className="font-medium">{metric.value}</span>
            </div>
          ))}
        </div>
      )}
      {progressBars && progressBars.length > 0 && (
        <div className="mt-3 space-y-2">
          {progressBars.map((bar, i) => (
            <div key={i} className="space-y-1">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">{bar.label}</span>
                <span className="font-medium">
                  {bar.value.toFixed(1)}
                  %
                </span>
              </div>
              <Progress value={bar.value} />
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
