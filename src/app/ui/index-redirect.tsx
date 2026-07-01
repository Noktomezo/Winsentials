import { Navigate } from '@tanstack/react-router'

export function IndexRedirect() {
  return <Navigate to="/home" replace />
}
