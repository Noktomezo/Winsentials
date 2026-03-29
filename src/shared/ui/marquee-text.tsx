import { useEffect, useRef, useState } from 'react'

interface MarqueeTextProps {
  text: string
  className?: string
}

export function MarqueeText({ text, className }: MarqueeTextProps) {
  const outerRef = useRef<HTMLSpanElement>(null)
  const innerRef = useRef<HTMLSpanElement>(null)
  const [offset, setOffset] = useState(0)

  useEffect(() => {
    if (!outerRef.current || !innerRef.current) {
      return
    }

    const outer = outerRef.current
    const inner = innerRef.current

    const measure = () => {
      requestAnimationFrame(() => {
        const diff = inner.scrollWidth - outer.offsetWidth
        setOffset(diff > 0 ? diff + 2 : 0)
      })
    }

    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(outer)
    return () => ro.disconnect()
  }, [text])

  return (
    <span
      className={`overflow-hidden ${className ?? ''}`}
      data-overflow={offset > 0 ? 'true' : 'false'}
      ref={outerRef}
    >
      <span
        className={offset > 0
          ? 'inline-block whitespace-nowrap transition-transform duration-700 ease-out will-change-transform motion-reduce:transition-none'
          : 'inline-block whitespace-nowrap'}
        data-marquee-inner="true"
        ref={innerRef}
        style={offset > 0 ? { ['--marquee-offset' as string]: `-${offset}px` } : undefined}
      >
        {text}
      </span>
    </span>
  )
}
