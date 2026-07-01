import { useEffect, useRef } from 'react'

export function useMountEffect(effect: () => void | (() => void)) {
  const effectRef = useRef(effect)
  effectRef.current = effect
  useEffect(() => effectRef.current(), [])
}
