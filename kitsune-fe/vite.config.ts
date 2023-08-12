import vue from '@vitejs/plugin-vue';

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
  plugins: [vue()],
});
