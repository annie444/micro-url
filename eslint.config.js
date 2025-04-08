import { defineConfig } from 'eslint/config'
import eslintPluginAstro from 'eslint-plugin-astro'
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended'
import jsxA11y from 'eslint-plugin-jsx-a11y'
import js from '@eslint/js'

export default defineConfig([
  config,
  js.configs.recommended,
  eslintPluginAstro.configs.recommended,
  jsxA11y.flatConfigs.recommended,
  eslintPluginPrettierRecommended,
  {
    rules: {
      semi: 'error',
      'prefer-const': 'error',
    },
  },
])
