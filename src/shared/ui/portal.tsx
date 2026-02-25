import type { ReactNode } from 'react'
import { useLayoutEffect, useRef } from 'react'
import { createPortal } from 'react-dom'

interface PortalProps {
  children: ReactNode
}

export function Portal({ children }: PortalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)

  useLayoutEffect(() => {
    const container = document.createElement('div')
    containerRef.current = container
    document.body.appendChild(container)

    return () => {
      document.body.removeChild(container)
    }
  }, [])

  if (!containerRef.current)
    return null

  return createPortal(children, containerRef.current)
}
