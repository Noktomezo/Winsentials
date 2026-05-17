import type { TFunction } from 'i18next'

interface FormatBytesOptions {
  decimals?: number
  locale: string
  t: TFunction
}

const UNIT_KEYS = [
  'format.byte',
  'format.kilobyte',
  'format.megabyte',
  'format.gigabyte',
  'format.terabyte',
] as const

const numberFormatCache = new Map<string, Intl.NumberFormat>()

function getNumberFormatter(locale: string, maximumFractionDigits: number) {
  const cacheKey = `${locale}:${maximumFractionDigits}`
  const cached = numberFormatCache.get(cacheKey)

  if (cached) {
    return cached
  }

  const formatter = Intl.NumberFormat(locale, { maximumFractionDigits })
  numberFormatCache.set(cacheKey, formatter)

  return formatter
}

export function formatBytesLocalized(bytes: number, { decimals = 1, locale, t }: FormatBytesOptions): string {
  if (bytes === 0) {
    return `0 ${t('format.byte')}`
  }

  const isNegative = bytes < 0
  const absBytes = Math.abs(bytes)
  const k = 1024
  const index = Math.min(Math.floor(Math.log(absBytes) / Math.log(k)), UNIT_KEYS.length - 1)
  const value = absBytes / k ** index
  const formatted = getNumberFormatter(locale, decimals).format(value)

  return `${isNegative ? '-' : ''}${formatted} ${t(UNIT_KEYS[index])}`
}

export function formatRateLocalized(bytesPerSec: number, options: FormatBytesOptions): string {
  return `${formatBytesLocalized(bytesPerSec, options)}${options.t('format.perSecond')}`
}
