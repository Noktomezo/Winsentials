import { flushSync } from 'react-dom'

type NavigationCallback = () => void | Promise<void>

let activeViewTransition: Promise<void> | null = null

export async function startRouteViewTransition(
  navigate: NavigationCallback,
): Promise<void> {
  if (activeViewTransition) {
    return activeViewTransition
  }

  const startViewTransition = document.startViewTransition?.bind(document)

  if (!startViewTransition) {
    await navigate()
    return
  }

  let navigationResult: void | Promise<void>

  const transition = startViewTransition(() => {
    // View Transitions needs the route tree swap flushed synchronously.
    // eslint-disable-next-line react-dom/no-flush-sync
    flushSync(() => {
      navigationResult = navigate()
    })

    return navigationResult
  })

  activeViewTransition = transition.finished.finally(() => {
    activeViewTransition = null
  })

  await activeViewTransition
}
