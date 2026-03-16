// @ts-check
import antfu from '@antfu/eslint-config'
import oxlint from 'eslint-plugin-oxlint'

export default antfu({
  react: true,
  ignores: [
    'AGENTS.md',
    'CLAUDE.md',
    'coverage',
    'dist',
    'node_modules',
    'src-tauri/gen/schemas',
    'src-tauri/target',
  ],
  lessOpinionated: true,
})
  .append(oxlint.configs['flat/recommended'])
  .append(
    {
      files: ['src/app/router.tsx', 'src/shared/ui/*.tsx'],
      rules: {
        'react-refresh/only-export-components': 'off',
      },
    },
    {
      files: ['src/shared/lib/hooks/use-mobile.ts'],
      rules: {
        'react-hooks-extra/no-direct-set-state-in-use-effect': 'off',
      },
    },
    {
      files: ['src/shared/ui/sidebar.tsx'],
      rules: {
        'react-naming-convention/use-state': 'off',
      },
    },
  )
