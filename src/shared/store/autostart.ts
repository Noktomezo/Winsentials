import type { AutostartItem } from '@/shared/types/autostart'

import { create } from 'zustand'
import {
  deleteAutostart,
  getAutostartItems,
  toggleAutostart,
} from '@/shared/api/autostart'

type Filter = 'all' | 'enabled' | 'disabled'

interface AutostartState {
  items: AutostartItem[]
  loading: boolean
  filter: Filter
  search: string
  load: () => Promise<void>
  toggle: (id: string, enable: boolean) => Promise<void>
  delete: (id: string) => Promise<void>
  setFilter: (filter: Filter) => void
  setSearch: (search: string) => void
}

export const useAutostartStore = create<AutostartState>((set, get) => ({
  items: [],
  loading: false,
  filter: 'all',
  search: '',

  load: async () => {
    set({ loading: true })
    try {
      const items = await getAutostartItems()
      set({ items, loading: false })
    }
    catch (error) {
      console.error('Failed to load autostart items:', error)
      set({ loading: false })
    }
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
