import type { AutostartItem } from '@/shared/types/autostart'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useOverlayScrollbars } from 'overlayscrollbars-react'
import { useEffect, useRef } from 'react'

import { AutostartRow } from './AutostartRow'

interface VirtualizedGroupProps {
  items: AutostartItem[]
  maxVisibleRows?: number
}

export function VirtualizedGroup({
  items,
  maxVisibleRows = 8,
}: VirtualizedGroupProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const viewportRef = useRef<HTMLDivElement>(null)

  const rowHeight = 58
  const rowGap = 8
  const maxHeight = (rowHeight + rowGap) * maxVisibleRows - rowGap

  const [initialize, osInstance] = useOverlayScrollbars({
    defer: false,
    options: {
      scrollbars: {
        theme: 'os-theme-winsentials',
        autoHide: 'never',
      },
    },
  })

  const isVirtualized = items.length > maxVisibleRows

  useEffect(() => {
    if (scrollRef.current && viewportRef.current) {
      initialize({
        target: scrollRef.current,
        elements: { viewport: viewportRef.current },
      })
    }

    return () => osInstance()?.destroy()
  }, [initialize, osInstance, isVirtualized])

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => viewportRef.current,
    estimateSize: () => rowHeight + rowGap,
    overscan: 3,
  })

  if (!isVirtualized) {
    return (
      <div className="space-y-2">
        {items.map(item => (
          <AutostartRow key={item.id} item={item} />
        ))}
      </div>
    )
  }

  const virtualItems = virtualizer.getVirtualItems()
  const totalSize = virtualizer.getTotalSize()

  return (
    <div ref={scrollRef} style={{ maxHeight: `${maxHeight}px` }}>
      <div ref={viewportRef}>
        <div
          style={{
            height: `${totalSize}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {virtualItems.map((virtualItem) => {
            const item = items[virtualItem.index]
            return (
              <div
                key={item.id}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                  paddingBottom: `${rowGap}px`,
                }}
              >
                <AutostartRow item={item} />
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
