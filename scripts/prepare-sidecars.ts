import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'
import process from 'node:process'

function sanitizeTargetTriple(value: string | undefined): string | undefined {
  if (!value) {
    return undefined
  }

  if (/^[\w.-]+$/.test(value) && !value.includes('..')) {
    return value
  }

  console.error(`Ignoring invalid TAURI_ENV_TARGET_TRIPLE: ${value}`)
  return undefined
}

const targetTriple = sanitizeTargetTriple(process.env.TAURI_ENV_TARGET_TRIPLE)
const targetDir = targetTriple
  ? join('src-tauri', 'target', targetTriple, 'release')
  : join('src-tauri', 'target', 'release')
const source = join(targetDir, 'winsentials_symlink_helper.exe')
const outputDir = join('src-tauri', 'binaries')
const outputName = targetTriple
  ? `winsentials_symlink_helper-${targetTriple}.exe`
  : 'winsentials_symlink_helper.exe'
const output = join(outputDir, outputName)

if (!existsSync(source)) {
  throw new Error(`Sidecar source not found: ${source}`)
}

mkdirSync(outputDir, { recursive: true })
copyFileSync(source, output)
console.log(`Prepared sidecar: ${output}`)
