import * as React from 'react'
import ReactDOM from 'react-dom/client'
import { initAppServices } from '@/app/init-app-services'
import { AppProviders } from '@/app/providers'
import { initPreferencesBootstrapCache } from '@/entities/settings/lib/bootstrap-cache'
import App from './App'
import '@fontsource/ibm-plex-sans/400.css'
import '@fontsource/ibm-plex-sans/500.css'
import '@fontsource/ibm-plex-sans/600.css'
import '@fontsource/ibm-plex-mono/400.css'
import '@fontsource/ibm-plex-mono/500.css'
import '@fontsource/ibm-plex-mono/600.css'
import '@/app/styles/index.css'
import '@/shared/i18n'

initPreferencesBootstrapCache()
initAppServices()

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AppProviders>
      <App />
    </AppProviders>
  </React.StrictMode>,
)
