export function mountToParam(mountPoint: string): string {
  return mountPoint.replace(/[:\\/]/g, '')
}

export function mountLabel(mountPoint: string): string {
  return mountPoint.replace(/[:\\/]/g, '').toUpperCase() || mountPoint
}
