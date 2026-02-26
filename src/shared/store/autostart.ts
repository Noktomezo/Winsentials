import type { AutostartItem, EnrichRequest } from '@/shared/types/autostart'

import { create } from 'zustand'
import {
  deleteAutostart,
  enrichAutostartItems,
  getAutostartItemsFast,
  toggleAutostart,
} from '@/shared/api/autostart'

type Filter = 'all' | 'enabled' | 'disabled'

interface AutostartState {
  items: AutostartItem[]
  loading: boolean
  enriching: boolean
  filter: Filter
  search: string
  loaded: boolean
  loadingPromise: Promise<void> | null
  enrichingPromise: Promise<void> | null

  load: () => Promise<void>
  fetchAndEnrich: () => Promise<void>
  enrich: () => Promise<void>
  forceReload: () => Promise<void>
  toggle: (id: string, enable: boolean) => Promise<void>
  delete: (id: string) => Promise<void>
  setFilter: (filter: Filter) => void
  setSearch: (search: string) => void
}

export const useAutostartStore = create<AutostartState>((set, get) => ({
  items: [],
  loading: false,
  enriching: false,
  filter: 'all',
  search: '',
  loaded: false,
  loadingPromise: null,
  enrichingPromise: null,

  load: async () => {
    if (get().loaded)
      return

    if (get().loadingPromise) {
      await get().loadingPromise
      return
    }

    const loadPromise = (async () => {
      set({ loading: true })
      try {
        const items = await getAutostartItemsFast()
        set({ items, loading: false, loaded: true })
        get().enrich()
      }
      catch (error) {
        console.error('Failed to load autostart items:', error)
        set({ loading: false })
      }
      finally {
        set({ loadingPromise: null })
      }
    })()

    set({ loadingPromise: loadPromise })
    await loadPromise
  },

  fetchAndEnrich: async () => {
    if (get().loadingPromise) {
      await get().loadingPromise
      return
    }

    const fetchPromise = (async () => {
      try {
        const items = await getAutostartItemsFast()
        set({ items, loading: false, loaded: true })
        get().enrich()
      }
      catch (error) {
        console.error('Failed to fetch autostart items:', error)
        set({ loading: false })
      }
      finally {
        set({ loadingPromise: null })
      }
    })()

    set({ loadingPromise: fetchPromise })
    await fetchPromise
  },

  enrich: async () => {
    if (get().enrichingPromise) {
      await get().enrichingPromise
      return
    }

    const enrichPromise = (async () => {
      set({ enriching: true })
      try {
        const requests: EnrichRequest[] = get().items.map(item => ({
          id: item.id,
          file_path: item.file_path,
        }))

        if (requests.length === 0) {
          set({ enriching: false })
          return
        }

        const enrichments = await enrichAutostartItems(requests)

        const enrichmentMap = new Map(enrichments.map(e => [e.id, e]))

        set(state => ({
          items: state.items.map((item) => {
            const enrichment = enrichmentMap.get(item.id)
            if (enrichment) {
              return {
                ...item,
                icon_base64: enrichment.icon_base64,
                publisher: enrichment.publisher,
              }
            }
            return item
          }),
          enriching: false,
        }))
      }
      catch (error) {
        console.error('Failed to enrich autostart items:', error)
        set({ enriching: false })
      }
      finally {
        set({ enrichingPromise: null })
      }
    })()

    set({ enrichingPromise: enrichPromise })
    await enrichPromise
  },

  forceReload: async () => {
    set({ loaded: false, loading: true })
    await get().fetchAndEnrich()
  },

  toggle: async (id: string, enable: boolean) => {
    await toggleAutostart(id, enable)
    const items = get().items.map(item =>
      item.id === id ? { ...item, is_enabled: enable } : item,
    )
    set({ items })
  },

  delete: async (id: string) => {
    await deleteAutostart(id)
    const items = get().items.filter(item => item.id !== id)
    set({ items })
  },

  setFilter: (filter: Filter) => set({ filter }),
  setSearch: (search: string) => set({ search }),
}))
