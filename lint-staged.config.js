const ignoredSegments = ['dist/', 'node_modules/', 'src-tauri/']

function normalizePath(file) {
  return file.replaceAll('\\', '/')
}

function quote(file) {
  return `"${file.replaceAll('"', '\\"')}"`
}

function filterFrontendFiles(files) {
  return files.filter((file) => {
    const normalized = normalizePath(file)

    return !ignoredSegments.some(segment => normalized.includes(segment.replace(/^\/+/, '')))
  })
}

export default {
  '**/*.{js,jsx,ts,tsx,cjs,mjs,json,md,yaml,yml}': (files) => {
    const eligibleFiles = filterFrontendFiles(files)

    if (eligibleFiles.length === 0) {
      return []
    }

    return [`bunx eslint --fix -- ${eligibleFiles.map(quote).join(' ')}`]
  },
}
