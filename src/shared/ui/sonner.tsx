import type { ResolvedTheme } from '@/shared/config/app'
import { Toaster as SonnerToaster } from 'sonner'

export function Toaster({ resolvedTheme }: { resolvedTheme: ResolvedTheme }) {
  return (
    <SonnerToaster
      position="bottom-right"
      theme={resolvedTheme}
      gap={8}
      toastOptions={{ duration: 4000 }}
    />
  )
}
