import type { ResolvedTheme } from '@/shared/config/app'
import { Toaster as SonnerToaster } from 'sonner'

export function Toaster({ resolvedTheme }: { resolvedTheme: ResolvedTheme }) {
  return (
    <SonnerToaster
      position="bottom-right"
      theme={resolvedTheme}
      gap={8}
      toastOptions={{
        duration: 4000,
        classNames: {
          actionButton: 'winsentials-sonner-action',
          cancelButton: 'winsentials-sonner-cancel',
          content: 'winsentials-sonner-content',
          description: 'winsentials-sonner-description',
          icon: 'winsentials-sonner-icon',
          title: 'winsentials-sonner-title',
          toast: 'winsentials-sonner-toast',
        },
      }}
    />
  )
}
