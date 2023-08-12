import vue from '@vitejs/plugin-vue';

import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          graphql: ['@vue/apollo-composable', 'graphql'],
          zxcvbnCommon: ['@zxcvbn-ts/language-common'],
          //zxcvbnEn: ['@zxcvbn-ts/language-en'],
        },
      },
    },
  },
  plugins: [vue()],
});
