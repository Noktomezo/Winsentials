import type { MotherboardInfo } from '@/entities/system-info/model/types'

const UNKNOWN_HARDWARE_VALUES = new Set([
  '',
  'default string',
  'none',
  'not applicable',
  'o.e.m.',
  'system product name',
  'to be filled by o.e.m.',
  'to be filled by oem',
  'unknown',
])

function cleanHardwareValue(value: string): string | null {
  const trimmedValue = value.trim()

  if (UNKNOWN_HARDWARE_VALUES.has(trimmedValue.toLowerCase())) {
    return null
  }

  return trimmedValue
}

function normalizeMotherboardManufacturer(value: string): string {
  const normalizedValue = value.toLowerCase()

  if (normalizedValue.includes('micro-star')) {
    return 'MSI'
  }

  if (normalizedValue.includes('asustek')) {
    return 'ASUS'
  }

  if (normalizedValue.includes('gigabyte')) {
    return 'GIGABYTE'
  }

  return value
}

export function formatMotherboardName(motherboard: MotherboardInfo): string | null {
  const manufacturer = cleanHardwareValue(motherboard.manufacturer)
  const product = cleanHardwareValue(motherboard.product)

  if (!manufacturer) {
    return product
  }

  const normalizedManufacturer = normalizeMotherboardManufacturer(manufacturer)

  if (!product) {
    return normalizedManufacturer
  }

  if (product.toLowerCase().includes(normalizedManufacturer.toLowerCase())) {
    return product
  }

  return `${normalizedManufacturer} ${product}`
}

export function formatCpuModel(model: string): string {
  return model
    .trim()
    .replace(/\s+\d+\s*(?:-\s*)?core\s+processor$/iu, '')
    .replace(/\s+cpu\s*@\s*[\d.]+\s*ghz$/iu, '')
    .trim()
}
