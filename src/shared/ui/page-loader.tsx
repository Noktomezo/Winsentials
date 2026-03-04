import { Skeleton } from '@/shared/ui/skeleton'

interface PageLoaderProps {
  isLoading: boolean
  skeleton?: React.ReactNode
  children: React.ReactNode
}

export function PageLoader({ isLoading, skeleton, children }: PageLoaderProps) {
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
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-24 w-full" />
        ))}
      </div>
    </div>
  )
}
