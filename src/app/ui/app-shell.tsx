import type { SmoothScrollAreaHandle } from '@/shared/ui/smooth-scroll-area'
import { useRouterState } from '@tanstack/react-router'
import { useRef } from 'react'
import { AppShellLayout } from '@/app/ui/app-shell-layout'
import { ScrollReset } from '@/app/ui/scroll-reset'

export function AppShell() {
  const pathname = useRouterState({
    select: state => state.location.pathname,
  })
  const scrollAreaRef = useRef<SmoothScrollAreaHandle>(null)

  return (
    <>
      <ScrollReset key={pathname} scrollAreaRef={scrollAreaRef} />
      <AppShellLayout pathname={pathname} scrollAreaRef={scrollAreaRef} />
    </>
  )
}
