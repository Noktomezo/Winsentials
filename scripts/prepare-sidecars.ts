import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'
import process from 'node:process'

const targetTriple = process.env.TAURI_ENV_TARGET_TRIPLE ?? 'x86_64-pc-windows-msvc'
const source = join('src-tauri', 'target', 'release', 'winsentials_symlink_helper.exe')
const outputDir = join('src-tauri', 'binaries')
const output = join(outputDir, `winsentials_symlink_helper-${targetTriple}.exe`)

if (!existsSync(source)) {
  throw new Error(`Sidecar source not found: ${source}`)
}

mkdirSync(outputDir, { recursive: true })
copyFileSync(source, output)
console.log(`Prepared sidecar: ${output}`)
