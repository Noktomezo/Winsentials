import type { TweakMeta, WindowsVersion } from '@/entities/tweak/model/types'
import { create } from 'zustand'
import { getTweaksByCategory, getWindowsBuild } from '@/entities/tweak/api'

export interface CachedTweakCategory {
  error: string | null
  fetchedAt: number | null
  hasLoaded: boolean
  isRefreshing: boolean
  tweaks: TweakMeta[]
}

interface TweakCacheState {
  categories: Record<string, CachedTweakCategory>
  ensureCategory: (category: string) => Promise<void>
  ensureWindowsBuild: () => Promise<WindowsVersion>
  revalidateCategory: (category: string) => Promise<void>
  setCategoryError: (category: string, error: string | null) => void
  updateCachedTweak: (category: string, id: string, currentValue: string) => void
  windowsBuild: WindowsVersion | null
  windowsBuildFetchedAt: number | null
}

export const EMPTY_CATEGORY: CachedTweakCategory = {
  error: null,
  fetchedAt: null,
  hasLoaded: false,
  isRefreshing: false,
  tweaks: [],
}

const categoryRequests = new Map<string, Promise<void>>()
let windowsBuildRequest: Promise<WindowsVersion> | null = null
const CATEGORY_REVALIDATE_TTL_MS = 30_000

export const useTweakCacheStore = create<TweakCacheState>()((set, get) => ({
  categories: {},
  async ensureCategory(category) {
    const cached = getCategorySnapshot(get().categories, category)

    if (cached.hasLoaded) {
      return
    }

    const inflight = categoryRequests.get(category)
    if (inflight) {
      return inflight
    }

    const request = fetchCategory(category, false)
      .finally(() => {
        categoryRequests.delete(category)
      })

    categoryRequests.set(category, request)
    return request
  },
  async ensureWindowsBuild() {
    const cached = get().windowsBuild

    if (cached) {
      return cached
    }

    if (windowsBuildRequest) {
      return windowsBuildRequest
    }

    windowsBuildRequest = getWindowsBuild()
      .then((build) => {
        set({
          windowsBuild: build,
          windowsBuildFetchedAt: Date.now(),
        })

        return build
      })
      .finally(() => {
        windowsBuildRequest = null
      })

    return windowsBuildRequest
  },
  async revalidateCategory(category) {
    const cached = getCategorySnapshot(get().categories, category)

    if (!cached.hasLoaded) {
      return get().ensureCategory(category)
    }

    if (
      cached.fetchedAt !== null
      && Date.now() - cached.fetchedAt < CATEGORY_REVALIDATE_TTL_MS
    ) {
      return
    }

    const inflight = categoryRequests.get(category)
    if (inflight) {
      return inflight
    }

    const request = fetchCategory(category, true)
      .catch((error) => {
        console.error(`Failed to revalidate tweak category "${category}"`, error)
      })
      .finally(() => {
        categoryRequests.delete(category)
      })

    categoryRequests.set(category, request)
    return request
  },
  setCategoryError(category, error) {
    setCategoryState(category, current => ({
      ...current,
      error,
    }))
  },
  updateCachedTweak(category, id, currentValue) {
    setCategoryState(category, current => ({
      ...current,
      tweaks: current.tweaks.map(tweak =>
        tweak.id === id ? { ...tweak, currentValue } : tweak,
      ),
    }))
  },
  windowsBuild: null,
  windowsBuildFetchedAt: null,
}))

function getCategorySnapshot(
  categories: Record<string, CachedTweakCategory>,
  category: string,
): CachedTweakCategory {
  return categories[category] ?? EMPTY_CATEGORY
}

function setCategoryState(
  category: string,
  updater: (current: CachedTweakCategory) => CachedTweakCategory,
) {
  useTweakCacheStore.setState((state) => {
    const current = getCategorySnapshot(state.categories, category)

    return {
      categories: {
        ...state.categories,
        [category]: updater(current),
      },
    }
  })
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error && error.message) {
    return error.message
  }

  if (typeof error === 'string' && error) {
    return error
  }

  return 'Failed to load tweaks.'
}

async function fetchCategory(category: string, isRefreshing: boolean) {
  const state = useTweakCacheStore.getState()
  const hasLoaded = getCategorySnapshot(state.categories, category).hasLoaded

  setCategoryState(category, current => ({
    ...current,
    error: hasLoaded ? current.error : null,
    isRefreshing,
  }))

  try {
    const [build, tweaks] = await Promise.all([
      state.ensureWindowsBuild(),
      getTweaksByCategory(category),
    ])

    useTweakCacheStore.setState(state => ({
      categories: {
        ...state.categories,
        [category]: {
          error: null,
          fetchedAt: Date.now(),
          hasLoaded: true,
          isRefreshing: false,
          tweaks,
        },
      },
      windowsBuild: build,
      windowsBuildFetchedAt: Date.now(),
    }))
  }
  catch (error) {
    setCategoryState(category, current => ({
      ...current,
      error: current.hasLoaded ? current.error : getErrorMessage(error),
      isRefreshing: false,
    }))

    throw error
  }
}
