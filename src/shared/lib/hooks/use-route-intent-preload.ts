import { useCallback } from 'react'

export function useRouteIntentPreload() {
  return useCallback((preload: () => Promise<unknown>) => {
    void preload().catch((error) => {
      console.warn('Failed to preload route', error)
    })
  }, [])
}
