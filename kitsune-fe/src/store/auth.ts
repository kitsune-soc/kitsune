import { defineStore } from 'pinia';
import { ref } from 'vue';

type TokenData = {
  token: string;
  refreshToken: string;
  expiresAt: Date;
};

export const useAuthStore = defineStore(
  'auth',
  () => {
    const data = ref<TokenData | undefined>(undefined);

    function isAuthenticated(): boolean {
      return data.value !== undefined;
    }

    return { data, isAuthenticated };
  },
  { persist: true },
);
