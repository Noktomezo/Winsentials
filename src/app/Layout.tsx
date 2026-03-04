import { OverlayScrollbarsComponent } from 'overlayscrollbars-react'
import { useEffect } from 'react'
import { useAutostartStore } from '@/shared/store/autostart'
import { TooltipProvider } from '@/shared/ui/tooltip'
import { Sidebar } from '@/widgets/sidebar'
import { Titlebar } from '@/widgets/titlebar'

interface LayoutProps {
  children: React.ReactNode
}

export function Layout({ children }: LayoutProps) {
  useEffect(() => {
    useAutostartStore.getState().load()
  }, [])

  return (
    <TooltipProvider>
      <div className="flex h-screen flex-col bg-background">
        <Titlebar />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />
          <OverlayScrollbarsComponent
            className="flex-1 p-6 os-theme-winsentials"
            defer
            options={{
              scrollbars: {
                theme: 'os-theme-winsentials',
                autoHide: 'never',
              },
            }}
          >
            {children}
          </OverlayScrollbarsComponent>
        </div>
      </div>
    </TooltipProvider>
  )
}
