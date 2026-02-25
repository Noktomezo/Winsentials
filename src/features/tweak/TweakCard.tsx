import type { TweakInfo } from '@/shared/types/tweak'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { AlertCircle, AlertTriangle, Info, RefreshCw, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { applyTweak, revertTweak } from '@/shared/api/tweaks'
import { Badge } from '@/shared/ui/badge'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import { RadioControl } from './RadioControl'
import { ToggleControl } from './ToggleControl'

interface TweakCardProps {
  tweak: TweakInfo
}

export function TweakCard({ tweak }: TweakCardProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const applyMutation = useMutation({
    mutationFn: (value: string | undefined) => applyTweak(tweak.meta.id, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tweaks', tweak.meta.category] })
    },
  })

  const revertMutation = useMutation({
    mutationFn: () => revertTweak(tweak.meta.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tweaks', tweak.meta.category] })
    },
  })

  const isLoading = applyMutation.isPending || revertMutation.isPending

  return (
    <div className="relative rounded-lg border border-border bg-card p-4">
      {!tweak.is_available && (
        <div className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-2 rounded-lg bg-background/80 backdrop-blur-sm">
          <AlertCircle className="h-5 w-5 text-muted-foreground" />
          <span className="text-sm font-medium text-muted-foreground">
            {t('tweak.unsupportedVersion')}
          </span>
        </div>
      )}
      <div className={`space-y-2 ${!tweak.is_available ? 'opacity-30' : ''}`}>
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <h3 className="truncate font-medium">
              {t(tweak.meta.name_key)}
            </h3>
            <div className="flex shrink-0 items-center gap-1">
              <Tooltip>
                <TooltipTrigger className="cursor-pointer">
                  <Badge variant="outline" className="border-muted-foreground/30 bg-muted/50 text-muted-foreground px-1.5">
                    <Info className="h-3 w-3" />
                  </Badge>
                </TooltipTrigger>
                <TooltipContent className="max-w-xs">
                  {t(tweak.meta.details_key)}
                </TooltipContent>
              </Tooltip>
              {tweak.meta.risk_level === 'medium' && (
                <Tooltip>
                  <TooltipTrigger className="cursor-pointer">
                    <Badge variant="outline" className="border-yellow-500/50 bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 px-1.5">
                      <AlertTriangle className="h-3 w-3" />
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    {t('tweak.riskLevel.medium')}
                  </TooltipContent>
                </Tooltip>
              )}
              {tweak.meta.risk_level === 'high' && (
                <Tooltip>
                  <TooltipTrigger className="cursor-pointer">
                    <Badge variant="outline" className="border-red-500/50 bg-red-500/10 text-red-600 dark:text-red-400 px-1.5">
                      <AlertTriangle className="h-3 w-3" />
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    {t('tweak.riskLevel.high')}
                  </TooltipContent>
                </Tooltip>
              )}
              {tweak.meta.requires_reboot && (
                <Tooltip>
                  <TooltipTrigger className="cursor-pointer">
                    <Badge variant="outline" className="border-blue-500/50 bg-blue-500/10 text-blue-600 dark:text-blue-400 px-1.5">
                      <RefreshCw className="h-3 w-3" />
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    {t('common.requiresReboot')}
                  </TooltipContent>
                </Tooltip>
              )}
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            {tweak.meta.ui_type === 'toggle' && (
              <ToggleControl
                tweak={tweak}
                onApply={() => applyMutation.mutate(undefined)}
                onRevert={() => revertMutation.mutate()}
                isLoading={isLoading}
              />
            )}
            {tweak.meta.ui_type === 'radio' && (
              <RadioControl
                tweak={tweak}
                onApply={value => applyMutation.mutate(value)}
                isLoading={isLoading}
              />
            )}
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  onClick={() => revertMutation.mutate()}
                  disabled={!tweak.state.is_applied || isLoading}
                  className="flex items-center justify-center rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                >
                  <RotateCcw className="h-4 w-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent>
                {t('common.revert')}
              </TooltipContent>
            </Tooltip>
          </div>
        </div>
        <p className="text-sm text-muted-foreground">
          {t(tweak.meta.description_key)}
        </p>
      </div>
    </div>
  )
}
