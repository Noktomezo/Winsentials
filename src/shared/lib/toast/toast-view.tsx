import type { ReactNode } from 'react'
import {
  CheckCircle2,
  Info,
  TriangleAlert,
  X,
  XCircle,
} from 'lucide-react'
import { cn } from '@/shared/lib/utils'

export interface ToastActionButton {
  label: ReactNode
  onClick: () => void | Promise<void>
}

export interface ToastCancelButton {
  label: ReactNode
  onClick?: () => void | Promise<void>
}

export type AppToastVariant = 'default' | 'error' | 'info' | 'success'

interface ToastViewProps {
  action?: ToastActionButton
  cancel?: ToastCancelButton
  description?: ReactNode
  extraActions?: ToastActionButton[]
  message: ReactNode
  onClose: () => void
  onCloseButton?: () => void | Promise<void>
  variant: AppToastVariant
  visible: boolean
}

function ToastIcon({ variant }: { variant: AppToastVariant }) {
  const className = cn(
    'toast-icon',
    variant === 'success' && 'toast-icon--success',
    variant === 'error' && 'toast-icon--error',
    variant === 'info' && 'toast-icon--info',
  )

  if (variant === 'success') {
    return <CheckCircle2 className={className} />
  }

  if (variant === 'error') {
    return <XCircle className={className} />
  }

  if (variant === 'info') {
    return <TriangleAlert className={className} />
  }

  return <Info className={className} />
}

export function ToastView({
  action,
  cancel,
  description,
  extraActions,
  message,
  onClose,
  onCloseButton,
  variant,
  visible,
}: ToastViewProps) {
  const handleButtonClick = (callback?: () => void | Promise<void>) => {
    const result = callback?.()
    onClose()

    if (result instanceof Promise) {
      void result
    }
  }

  return (
    <div
      className={cn(
        'toast-root',
        visible ? 'toast-root--visible' : 'toast-root--hidden',
        variant === 'success' && 'toast-root--success',
        variant === 'error' && 'toast-root--error',
        variant === 'info' && 'toast-root--info',
      )}
    >
      <div className="toast-main">
        <div className="toast-icon-wrap">
          <ToastIcon variant={variant} />
        </div>
        <div className="toast-content">
          <div className="toast-title">{message}</div>
          {description && (
            <div className="toast-description">{description}</div>
          )}
        </div>
        <button
          aria-label="Close notification"
          className="toast-close"
          onClick={() => {
            const result = onCloseButton?.()
            onClose()

            if (result instanceof Promise) {
              void result
            }
          }}
          type="button"
        >
          <X className="size-4" />
        </button>
      </div>
      {(action || cancel || extraActions?.length) && (
        <div className="toast-actions">
          {cancel && (
            <button
              className="toast-cancel"
              onClick={() => handleButtonClick(cancel.onClick)}
              type="button"
            >
              {cancel.label}
            </button>
          )}
          {extraActions?.map(extraAction => (
            <button
              key={String(extraAction.label)}
              className="toast-cancel"
              onClick={() => handleButtonClick(extraAction.onClick)}
              type="button"
            >
              {extraAction.label}
            </button>
          ))}
          {action && (
            <button
              className="toast-action"
              onClick={() => handleButtonClick(action.onClick)}
              type="button"
            >
              {action.label}
            </button>
          )}
        </div>
      )}
    </div>
  )
}
