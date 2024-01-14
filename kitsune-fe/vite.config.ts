import vue from '@vitejs/plugin-vue';

import path from 'node:path';
import { ExternalFluentPlugin } from 'unplugin-fluent-vue/vite';
import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          zxcvbnCommon: ['@zxcvbn-ts/language-common'],
        },
      },
    },
  },
  plugins: [
    vue(),
    ExternalFluentPlugin({
      locales: ['en', 'en-cyberpunk'],
      checkSyntax: true,

      baseDir: path.resolve('src'),
      ftlDir: path.resolve('src/locales'),
    }),
  ],
});
