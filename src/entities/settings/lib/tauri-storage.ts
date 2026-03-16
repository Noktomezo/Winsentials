import type { StateStorage } from 'zustand/middleware'
import { LazyStore } from '@tauri-apps/plugin-store'

const PREFERENCES_STORE_FILE = 'settings.json'

const store = new LazyStore(PREFERENCES_STORE_FILE)

export const tauriStateStorage: StateStorage = {
  async getItem(name) {
    const storedValue = await store.get<string>(name)

    if (typeof storedValue === 'string') {
      return storedValue
    }

    const legacyValue = window.localStorage.getItem(name)

    if (legacyValue === null) {
      return null
    }

    await store.set(name, legacyValue)
    await store.save()
    window.localStorage.removeItem(name)

    return legacyValue
  },
  async removeItem(name) {
    await store.delete(name)
    await store.save()
    window.localStorage.removeItem(name)
  },
  async setItem(name, value) {
    await store.set(name, value)
    await store.save()
  },
}
