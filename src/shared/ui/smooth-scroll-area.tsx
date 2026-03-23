import type {
  MouseEvent,
  PropsWithChildren,
  PointerEvent as ReactPointerEvent,
} from 'react'
import Lenis from 'lenis'
import { useImperativeHandle, useRef, useState } from 'react'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { cn } from '@/shared/lib/utils'

interface SmoothScrollAreaProps extends PropsWithChildren {
  className?: string
}

export interface SmoothScrollAreaHandle {
  scrollTo: (y: number, immediate?: boolean) => void
  scrollToTop: (immediate?: boolean) => void
}

const MIN_THUMB_HEIGHT = 32

export const SmoothScrollArea = function SmoothScrollArea({ ref, children, className }: SmoothScrollAreaProps & { ref?: React.RefObject<SmoothScrollAreaHandle | null> }) {
  const lenisRef = useRef<Lenis | null>(null)
  const thumbRef = useRef<HTMLDivElement | null>(null)
  const trackRef = useRef<HTMLDivElement | null>(null)
  const wrapperRef = useRef<HTMLDivElement | null>(null)
  const contentRef = useRef<HTMLDivElement | null>(null)
  const dragStateRef = useRef<{
    startPointerY: number
    startScrollTop: number
  } | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  useMountEffect(() => {
    const wrapper = wrapperRef.current
    const content = contentRef.current
    const track = trackRef.current
    const thumb = thumbRef.current

    if (!wrapper || !content || !track || !thumb) {
      return
    }

    const lenis = new Lenis({
      autoRaf: false,
      content,
      gestureOrientation: 'vertical',
      lerp: 0.08,
      orientation: 'vertical',
      smoothWheel: true,
      syncTouch: false,
      wheelMultiplier: 0.95,
      wrapper,
    })

    lenisRef.current = lenis

    const updateScrollbar = () => {
      const clientHeight = wrapper.clientHeight
      const scrollHeight = content.scrollHeight
      const maxScroll = scrollHeight - clientHeight
      const nextScrollable = maxScroll > 0

      track.style.display = nextScrollable ? 'block' : 'none'
      track.style.pointerEvents = nextScrollable ? 'auto' : 'none'

      if (!nextScrollable) {
        thumb.style.height = '0px'
        thumb.style.transform = 'translate3d(0, 0, 0)'
        return
      }

      const trackHeight = track.clientHeight
      const thumbHeight = Math.max((clientHeight / scrollHeight) * trackHeight, MIN_THUMB_HEIGHT)
      const maxThumbOffset = Math.max(trackHeight - thumbHeight, 0)
      const scrollProgress = maxScroll > 0 ? wrapper.scrollTop / maxScroll : 0
      const thumbOffset = scrollProgress * maxThumbOffset

      thumb.style.height = `${thumbHeight}px`
      thumb.style.transform = `translate3d(0, ${thumbOffset}px, 0)`
    }

    let frameId = 0

    const onFrame = (time: number) => {
      lenis.raf(time)
      frameId = window.requestAnimationFrame(onFrame)
    }

    const resizeObserver = new ResizeObserver(() => {
      lenis.resize()
      updateScrollbar()
    })

    resizeObserver.observe(wrapper)
    resizeObserver.observe(content)
    lenis.on('scroll', updateScrollbar)
    updateScrollbar()
    frameId = window.requestAnimationFrame(onFrame)

    return () => {
      resizeObserver.disconnect()
      window.cancelAnimationFrame(frameId)
      lenis.destroy()
      lenisRef.current = null
    }
  })

  const scrollTo = (targetScrollTop: number, immediate = false) => {
    lenisRef.current?.scrollTo(targetScrollTop, immediate
      ? { immediate: true }
      : { duration: 0.45 })
  }

  useImperativeHandle(ref, () => ({
    scrollTo: (y: number, immediate = false) => {
      scrollTo(y, immediate)
    },
    scrollToTop: (immediate = false) => {
      scrollTo(0, immediate)
    },
  }))

  const handleThumbPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    const wrapper = wrapperRef.current
    const track = trackRef.current
    const thumb = thumbRef.current

    if (!wrapper || !track || !thumb) {
      return
    }

    event.preventDefault()
    event.stopPropagation()

    dragStateRef.current = {
      startPointerY: event.clientY,
      startScrollTop: wrapper.scrollTop,
    }

    setIsDragging(true)
    event.currentTarget.setPointerCapture(event.pointerId)

    const previousUserSelect = document.body.style.userSelect
    document.body.style.userSelect = 'none'

    const handlePointerMove = (pointerEvent: PointerEvent) => {
      const dragState = dragStateRef.current

      if (!dragState) {
        return
      }

      const maxScroll = contentRef.current
        ? contentRef.current.scrollHeight - wrapper.clientHeight
        : 0
      const trackHeight = track.clientHeight
      const thumbHeight = thumb.offsetHeight
      const maxThumbOffset = trackHeight - thumbHeight

      if (maxScroll <= 0 || maxThumbOffset <= 0) {
        return
      }

      const deltaY = pointerEvent.clientY - dragState.startPointerY
      const scrollDelta = (deltaY / maxThumbOffset) * maxScroll
      scrollTo(dragState.startScrollTop + scrollDelta, true)
    }

    const handlePointerEnd = () => {
      dragStateRef.current = null
      setIsDragging(false)
      document.body.style.userSelect = previousUserSelect
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', handlePointerEnd)
      window.removeEventListener('pointercancel', handlePointerEnd)
    }

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerEnd)
    window.addEventListener('pointercancel', handlePointerEnd)
  }

  const handleTrackClick = (event: MouseEvent<HTMLDivElement>) => {
    const wrapper = wrapperRef.current
    const content = contentRef.current
    const track = trackRef.current
    const thumb = thumbRef.current

    if (!wrapper || !content || !track || !thumb) {
      return
    }

    if (event.target === thumb) {
      return
    }

    const maxScroll = content.scrollHeight - wrapper.clientHeight

    if (maxScroll <= 0) {
      return
    }

    const rect = track.getBoundingClientRect()
    const thumbHeight = thumb.offsetHeight
    const maxThumbOffset = rect.height - thumbHeight

    if (maxThumbOffset <= 0) {
      return
    }

    const rawOffset = event.clientY - rect.top - thumbHeight / 2
    const clampedOffset = Math.min(Math.max(rawOffset, 0), maxThumbOffset)
    const progress = clampedOffset / maxThumbOffset

    scrollTo(progress * maxScroll)
  }

  return (
    <div
      className={cn('group/scroll relative h-full overflow-hidden', className)}
      data-dragging={isDragging ? 'true' : 'false'}
    >
      <div ref={wrapperRef} className="scrollbar-hidden h-full overflow-auto">
        <div ref={contentRef} className="min-h-full">
          {children}
        </div>
      </div>
      <div
        ref={trackRef}
        className={cn(
          'absolute inset-y-0 right-0 z-10 w-2.5 p-px transition-opacity duration-200',
          'opacity-0 group-hover/scroll:opacity-100',
          isDragging && 'opacity-100',
        )}
        onClick={handleTrackClick}
      >
        <div
          ref={thumbRef}
          className={cn(
            'pointer-events-auto relative w-full rounded-full bg-border/70 transition-colors duration-200',
            'hover:bg-border',
            isDragging && 'bg-primary/60',
          )}
          onPointerDown={handleThumbPointerDown}
        />
      </div>
    </div>
  )
}
