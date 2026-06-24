import type { RefObject } from 'react'
import type { SmoothScrollAreaHandle } from '@/shared/ui/smooth-scroll-area'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'

export function ScrollReset({ scrollAreaRef }: { scrollAreaRef: RefObject<SmoothScrollAreaHandle | null> }) {
  useMountEffect(() => {
    scrollAreaRef.current?.scrollToTop(true)
  })

  return null
}
