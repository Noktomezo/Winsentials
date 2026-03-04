import type { ReactNode } from 'react'
import { Skeleton } from '@/shared/ui/skeleton'

interface PageLoaderProps {
  isLoading: boolean
  skeleton?: ReactNode
  children: ReactNode
  rows?: number
}

export function PageLoader({ isLoading, skeleton, children, rows = 5 }: PageLoaderProps) {
  if (!isLoading) {
    return <>{children}</>
  }

  if (skeleton) {
    return <>{skeleton}</>
  }

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-64" />
      </div>
      <div className="space-y-4">
        {Array.from({ length: rows }).map((_, i) => (
          <Skeleton key={i} className="h-24 w-full" />
        ))}
      </div>
    </div>
  )
}
