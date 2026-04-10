// @ts-check
import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: [
    'AGENTS.md',
    'CLAUDE.md',
    'coverage',
    'dist',
    'node_modules',
    'src-tauri',
  ],
  lessOpinionated: true,
  formatters: true,
}).append({
  files: ['**/*.{js,jsx,ts,tsx}'],
  rules: {
    'e18e/prefer-static-regex': 'off',
    'curly': 'off',
  },
})
