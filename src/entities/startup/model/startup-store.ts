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
  hydrateStartupEntries,
} from '@/entities/startup/api'

interface StartupStoreState {
  entries: StartupEntry[]
  entriesBySource: Record<StartupSource, StartupEntry[]>
  sourceLoading: Record<StartupSource, boolean>
  sourceErrors: Record<StartupSource, string | null>
  hasLoadedSource: Record<StartupSource, boolean>
  hasSettledSource: Record<StartupSource, boolean>
  sourceRequestIds: Record<StartupSource, number>
  hydrationRequestId: number
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

const emptySettledState: Record<StartupSource, boolean> = {
  registry: false,
  startup_folder: false,
  scheduled_task: false,
}

const emptySourceRequestIds: Record<StartupSource, number> = {
  registry: 0,
  startup_folder: 0,
  scheduled_task: 0,
}

const hydrationChunkSize = 12

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

function applyEntryUpdate(
  current: Record<StartupSource, StartupEntry[]>,
  entry: StartupEntry,
) {
  const next = {
    ...current,
    [entry.source]: current[entry.source].map(currentEntry => currentEntry.id === entry.id ? entry : currentEntry),
  }

  return {
    entriesBySource: next,
    entries: mergeEntries(next),
  }
}

function applyEntryUpdates(
  current: Record<StartupSource, StartupEntry[]>,
  entries: StartupEntry[],
) {
  if (entries.length === 0) {
    return {
      entriesBySource: current,
      entries: mergeEntries(current),
    }
  }

  const next = { ...current }

  for (const entry of entries) {
    next[entry.source] = next[entry.source].map(currentEntry => currentEntry.id === entry.id ? entry : currentEntry)
  }

  return {
    entriesBySource: next,
    entries: mergeEntries(next),
  }
}

function removeEntry(
  current: Record<StartupSource, StartupEntry[]>,
  source: StartupSource,
  id: string,
) {
  const next = {
    ...current,
    [source]: current[source].filter(entry => entry.id !== id),
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
        hasSettledSource: {
          ...current.hasSettledSource,
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
        hasSettledSource: {
          ...current.hasSettledSource,
          [source]: true,
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

async function hydrateLoadedEntries(
  requestId: number,
  set: StartupStoreSetter,
  get: () => StartupStoreState,
) {
  const ids = get().entries.filter(entry => entry.iconDataUrl === null || entry.publisher === null).map(entry => entry.id)

  for (let index = 0; index < ids.length; index += hydrationChunkSize) {
    if (get().hydrationRequestId !== requestId) {
      return
    }

    const chunk = ids.slice(index, index + hydrationChunkSize)
    let hydrated: StartupEntry[] = []

    try {
      hydrated = await hydrateStartupEntries(chunk)
    }
    catch (error) {
      console.error('Failed to hydrate startup entries chunk', error)
      continue
    }

    if (get().hydrationRequestId !== requestId || hydrated.length === 0) {
      continue
    }

    set((current) => {
      if (current.hydrationRequestId !== requestId) {
        return current
      }

      return {
        ...current,
        ...applyEntryUpdates(current.entriesBySource, hydrated),
      }
    })
  }
}

export const useStartupStore = create<StartupStoreState>()((set, get) => ({
  entries: [],
  entriesBySource: emptyEntriesBySource,
  sourceLoading: emptySourceLoading,
  sourceErrors: emptySourceErrors,
  hasLoadedSource: emptyLoadedState,
  hasSettledSource: emptySettledState,
  sourceRequestIds: emptySourceRequestIds,
  hydrationRequestId: 0,
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
    const requestId = get().hydrationRequestId + 1
    set({ hydrationRequestId: requestId })

    await Promise.allSettled([
      refreshSource('registry', false, set, get),
      refreshSource('startup_folder', false, set, get),
    ])

    await refreshSource('scheduled_task', false, set, get)
    void hydrateLoadedEntries(requestId, set, get)
  },
  async enableEntry(id) {
    set(state => ({ pendingIds: pushPending(state.pendingIds, id) }))
    try {
      const entry = await enableStartupEntry(id)
      set(current => ({
        ...current,
        ...applyEntryUpdate(current.entriesBySource, entry),
      }))
    }
    finally {
      set(state => ({ pendingIds: popPending(state.pendingIds, id) }))
    }
  },
  async disableEntry(id) {
    set(state => ({ pendingIds: pushPending(state.pendingIds, id) }))
    try {
      const entry = await disableStartupEntry(id)
      set(current => ({
        ...current,
        ...applyEntryUpdate(current.entriesBySource, entry),
      }))
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
        set(current => ({
          ...current,
          ...removeEntry(current.entriesBySource, source, id),
        }))
      }
    }
    finally {
      set(state => ({ pendingIds: popPending(state.pendingIds, id) }))
    }
  },
}))
