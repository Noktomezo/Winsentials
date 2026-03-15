import type { ReactNode } from 'react'
import { toast as hotToast } from 'react-hot-toast'
import {
  ToastView,
} from '@/shared/lib/toast/toast-view'

export interface ToastMessageOptions {
  description?: ReactNode
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
  extraActions?: Array<{
    label: ReactNode
    onClick: () => void | Promise<void>
  }>
  onCloseButton?: () => void | Promise<void>
}

const DEFAULT_DURATION = 4000
const ACTION_DURATION = 8000
const REMOVE_DELAY = 220

function showToast(
  variant: 'default' | 'error' | 'info' | 'success',
  message: ReactNode,
  options: ToastMessageOptions & {
    action?: ToastActionOptions['action']
    cancel?: ToastActionOptions['cancel']
    extraActions?: ToastActionOptions['extraActions']
    onCloseButton?: ToastActionOptions['onCloseButton']
  } = {},
) {
  return hotToast.custom(
    current => (
      <ToastView
        action={options.action}
        cancel={options.cancel}
        description={options.description}
        extraActions={options.extraActions}
        message={message}
        onClose={() => hotToast.dismiss(current.id)}
        onCloseButton={options.onCloseButton}
        variant={variant}
        visible={current.visible}
      />
    ),
    {
      duration: options.action || options.cancel
        ? ACTION_DURATION
        : (options.duration ?? DEFAULT_DURATION),
      position: 'bottom-right',
      removeDelay: REMOVE_DELAY,
    },
  )
}

export const toast = {
  action(message: ReactNode, options: ToastActionOptions) {
    return showToast('info', message, options)
  },
  dismiss(id?: string) {
    hotToast.dismiss(id)
  },
  error(message: ReactNode, options?: ToastMessageOptions) {
    return showToast('error', message, options)
  },
  message(message: ReactNode, options?: ToastMessageOptions) {
    return showToast('default', message, options)
  },
  success(message: ReactNode, options?: ToastMessageOptions) {
    return showToast('success', message, options)
  },
}
