export function mountToParam(mountPoint: string): string {
  return mountPoint.replace(/[:\\/]/g, '')
}

export function mountLabel(mountPoint: string): string {
  return mountToParam(mountPoint).toUpperCase() || mountPoint
}

export function networkAdapterToParam(name: string): string {
  return name
}
