import { defineStore } from 'pinia';
import { ref } from 'vue';

import { refreshAccessToken } from '../lib/oauth2';

export type TokenData = {
  token: string;
  refreshToken: string;
  expiresAt: string; // This has to be a string because for some reason the persisted state plugin struggles with dates a bit..
};

export const useAuthStore = defineStore(
  'auth',
  () => {
    const data = ref<TokenData | undefined>(undefined);

    function isAuthenticated(): boolean {
      return data.value !== undefined;
    }

    async function accessToken(): Promise<string | null> {
      if (!isAuthenticated()) {
        return null;
      }

      if (new Date(data.value!.expiresAt) > new Date()) {
        return data.value!.token;
      }

      return (await refreshAccessToken()).token;
    }

    return { accessToken, data, isAuthenticated };
  },
  { persist: true },
);
