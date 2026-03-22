import type {
  StartupEntry,
  StartupSource,
  StartupSourceFilter,
  StartupSourceListResponse,
  StartupStatusFilter,
} from '@/entities/startup/model/types'
import { create } from 'zustand'
import {
  deleteStartupEntry,
  disableStartupEntry,
  enableStartupEntry,
  getRegistryStartupEntries,
  getScheduledTaskStartupEntries,
  getStartupFolderEntries,
} from '@/entities/startup/api'

interface StartupStoreState {
  entries: StartupEntry[]
  entriesBySource: Record<StartupSource, StartupEntry[]>
  sourceLoading: Record<StartupSource, boolean>
  sourceErrors: Record<StartupSource, string | null>
  hasLoadedSource: Record<StartupSource, boolean>
  sourceRequestIds: Record<StartupSource, number>
  search: string
  sourceFilter: StartupSourceFilter
  statusFilter: StartupStatusFilter
  pendingIds: string[]
  error: string | null
  setSearch: (search: string) => void
  setSourceFilter: (filter: StartupSourceFilter) => void
  setStatusFilter: (filter: StartupStatusFilter) => void
  loadRegistryEntries: (force?: boolean) => Promise<void>
  loadStartupFolderEntries: (force?: boolean) => Promise<void>
  loadScheduledTaskEntries: (force?: boolean) => Promise<void>
  loadAllEntriesProgressive: () => Promise<void>
  enableEntry: (id: string) => Promise<void>
  disableEntry: (id: string) => Promise<void>
  deleteEntry: (id: string) => Promise<void>
}

type StartupStoreSetter = (
  partial: StartupStoreState
    | Partial<StartupStoreState>
    | ((state: StartupStoreState) => StartupStoreState | Partial<StartupStoreState>),
) => void

const startupSources = ['registry', 'startup_folder', 'scheduled_task'] as const satisfies readonly StartupSource[]

const emptyEntriesBySource: Record<StartupSource, StartupEntry[]> = {
  registry: [],
  startup_folder: [],
  scheduled_task: [],
}

const emptySourceLoading: Record<StartupSource, boolean> = {
  registry: false,
  startup_folder: false,
  scheduled_task: false,
}

const emptySourceErrors: Record<StartupSource, string | null> = {
  registry: null,
  startup_folder: null,
  scheduled_task: null,
}

const emptyLoadedState: Record<StartupSource, boolean> = {
  registry: false,
  startup_folder: false,
  scheduled_task: false,
}

const emptySourceRequestIds: Record<StartupSource, number> = {
  registry: 0,
  startup_folder: 0,
  scheduled_task: 0,
}

function pushPending(current: string[], id: string) {
  return current.includes(id) ? current : [...current, id]
}

function popPending(current: string[], id: string) {
  return current.filter(entryId => entryId !== id)
}

function sourceRank(source: StartupSource) {
  switch (source) {
    case 'registry':
      return 0
    case 'startup_folder':
      return 1
    case 'scheduled_task':
      return 2
  }
}

function mergeEntries(entriesBySource: Record<StartupSource, StartupEntry[]>) {
  return startupSources
    .flatMap(source => entriesBySource[source])
    .sort((left, right) => {
      return sourceRank(left.source) - sourceRank(right.source)
        || left.displayName.localeCompare(right.displayName, undefined, { sensitivity: 'base' })
    })
}

function applySourceResponse(
  current: Record<StartupSource, StartupEntry[]>,
  response: StartupSourceListResponse,
) {
  const next = {
    ...current,
    [response.source]: response.entries,
  }

  return {
    entriesBySource: next,
    entries: mergeEntries(next),
  }
}

function sourceFromId(id: string): StartupSource | null {
  if (id.startsWith('reg:')) {
    return 'registry'
  }

  if (id.startsWith('folder:')) {
    return 'startup_folder'
  }

  if (id.startsWith('task:')) {
    return 'scheduled_task'
  }

  return null
}

async function refreshSource(
  source: StartupSource,
  force: boolean,
  set: StartupStoreSetter,
  get: () => StartupStoreState,
) {
  const state = get()
  if (!force && (state.sourceLoading[source] || state.hasLoadedSource[source])) {
    return
  }

  let requestId = 0
  const previousHasLoaded = state.hasLoadedSource[source]
  set(current => ({
    ...current,
    error: null,
    sourceLoading: {
      ...current.sourceLoading,
      [source]: true,
    },
    sourceRequestIds: {
      ...current.sourceRequestIds,
      [source]: (requestId = current.sourceRequestIds[source] + 1),
    },
  }))

  try {
    const response = await fetchSource(source)
    set((current) => {
      if (current.sourceRequestIds[source] !== requestId) {
        return current
      }

      return {
        ...current,
        ...applySourceResponse(current.entriesBySource, response),
        sourceErrors: {
          ...current.sourceErrors,
          [source]: response.error,
        },
        hasLoadedSource: {
          ...current.hasLoadedSource,
          [source]: true,
        },
      }
    })
  }
  catch (error) {
    set((current) => {
      if (current.sourceRequestIds[source] !== requestId) {
        return current
      }

      return {
        ...current,
        sourceErrors: {
          ...current.sourceErrors,
          [source]: error instanceof Error ? error.message : 'Unknown startup source error.',
        },
        hasLoadedSource: {
          ...current.hasLoadedSource,
          [source]: previousHasLoaded,
        },
        error: error instanceof Error ? error.message : 'Failed to load startup entries.',
      }
    })
  }
  finally {
    set((current) => {
      if (current.sourceRequestIds[source] !== requestId) {
        return current
      }

      return {
        ...current,
        sourceLoading: {
          ...current.sourceLoading,
          [source]: false,
        },
      }
    })
  }
}

function fetchSource(source: StartupSource) {
  switch (source) {
    case 'registry':
      return getRegistryStartupEntries()
    case 'startup_folder':
      return getStartupFolderEntries()
    case 'scheduled_task':
      return getScheduledTaskStartupEntries()
  }
}

export const useStartupStore = create<StartupStoreState>()((set, get) => ({
  entries: [],
  entriesBySource: emptyEntriesBySource,
  sourceLoading: emptySourceLoading,
  sourceErrors: emptySourceErrors,
  hasLoadedSource: emptyLoadedState,
  sourceRequestIds: emptySourceRequestIds,
  search: '',
  sourceFilter: 'all',
  statusFilter: 'all',
  pendingIds: [],
  error: null,
  setSearch: search => set({ search }),
  setSourceFilter: sourceFilter => set({ sourceFilter }),
  setStatusFilter: statusFilter => set({ statusFilter }),
  async loadRegistryEntries(force = false) {
    await refreshSource('registry', force, set, get)
  },
  async loadStartupFolderEntries(force = false) {
    await refreshSource('startup_folder', force, set, get)
  },
  async loadScheduledTaskEntries(force = false) {
    await refreshSource('scheduled_task', force, set, get)
  },
  async loadAllEntriesProgressive() {
    const registryPromise = refreshSource('registry', false, set, get)
    const startupFolderPromise = refreshSource('startup_folder', false, set, get)
    const scheduledTasksPromise = refreshSource('scheduled_task', false, set, get)

    await Promise.allSettled([
      registryPromise,
      startupFolderPromise,
      scheduledTasksPromise,
    ])
  },
  async enableEntry(id) {
    const source = sourceFromId(id)
    set(state => ({ pendingIds: pushPending(state.pendingIds, id) }))
    try {
      await enableStartupEntry(id)
      if (source) {
        await refreshSource(source, true, set, get)
      }
    }
    finally {
      set(state => ({ pendingIds: popPending(state.pendingIds, id) }))
    }
  },
  async disableEntry(id) {
    const source = sourceFromId(id)
    set(state => ({ pendingIds: pushPending(state.pendingIds, id) }))
    try {
      await disableStartupEntry(id)
      if (source) {
        await refreshSource(source, true, set, get)
      }
    }
    finally {
      set(state => ({ pendingIds: popPending(state.pendingIds, id) }))
    }
  },
  async deleteEntry(id) {
    const source = sourceFromId(id)
    set(state => ({ pendingIds: pushPending(state.pendingIds, id) }))
    try {
      await deleteStartupEntry(id)
      if (source) {
        await refreshSource(source, true, set, get)
      }
    }
    finally {
      set(state => ({ pendingIds: popPending(state.pendingIds, id) }))
    }
  },
}))
