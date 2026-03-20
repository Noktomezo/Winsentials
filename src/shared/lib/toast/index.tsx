import type { ReactNode } from 'react'
import { toast as sonner } from 'sonner'

export interface ToastMessageOptions {
  description?: string
  duration?: number
}

export interface ToastActionOptions extends ToastMessageOptions {
  action: {
    label: ReactNode
    onClick: () => void | Promise<void>
  }
  cancel?: {
    label: ReactNode
    onClick?: () => void | Promise<void>
  }
}

export const toast = {
  action(message: string, options: ToastActionOptions) {
    return sonner(message, {
      description: options.description,
      duration: options.duration ?? 8000,
      action: {
        label: options.action.label,
        onClick: () => { void options.action.onClick() },
      },
      cancel: options.cancel
        ? {
            label: options.cancel.label,
            onClick: () => { void options.cancel!.onClick?.() },
          }
        : undefined,
    })
  },
  dismiss(id?: string | number) {
    sonner.dismiss(id)
  },
  error(message: string, options?: ToastMessageOptions) {
    return sonner.error(message, {
      description: options?.description,
      duration: options?.duration ?? 4000,
    })
  },
  message(message: string, options?: ToastMessageOptions) {
    return sonner(message, {
      description: options?.description,
      duration: options?.duration ?? 4000,
    })
  },
  success(message: string, options?: ToastMessageOptions) {
    return sonner.success(message, {
      description: options?.description,
      duration: options?.duration ?? 4000,
    })
  },
}
