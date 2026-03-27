import type { PropsWithChildren } from 'react'
import { useResolvedTheme } from '@/shared/lib/hooks/use-resolved-theme'
import { Toaster } from '@/shared/ui/sonner'
import { TooltipProvider } from '@/shared/ui/tooltip'

export function AppProviders({ children }: PropsWithChildren) {
  const resolvedTheme = useResolvedTheme()

  return (
    <TooltipProvider delayDuration={150}>
      {children}
      <Toaster resolvedTheme={resolvedTheme} />
    </TooltipProvider>
  )
}
