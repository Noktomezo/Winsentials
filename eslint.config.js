// @ts-check
import antfu from '@antfu/eslint-config'
import oxlint from 'eslint-plugin-oxlint'

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
})
  .append(oxlint.configs['flat/recommended'])
  .append({
    files: ['**/*.{js,jsx,ts,tsx}'],
    rules: {
      'style/max-statements-per-line': 'off',
      'e18e/prefer-static-regex': 'off',
    },
  })
