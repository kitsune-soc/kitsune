import { FlatCompat } from '@eslint/eslintrc';
import js from '@eslint/js';

import eslintVue from 'eslint-plugin-vue';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const compat = new FlatCompat({
  baseDirectory: path.dirname(fileURLToPath(import.meta.url)),
  recommendedConfig: js.configs.recommended,
});

export default [
  ...eslintVue.configs['flat/recommended'],
  ...compat.extends('@vue/eslint-config-prettier/skip-formatting'),
  ...compat.extends('@vue/eslint-config-typescript/recommended'),
  {
    ignores: ['.gitignore', 'src/graphql/types/**'],
  },
];
