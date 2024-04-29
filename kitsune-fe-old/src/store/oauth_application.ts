import { defineStore } from 'pinia';
import { ref } from 'vue';

export type OAuthApplication = {
  id: string;
  secret: string;
  redirectUri: string;
};

export const useOAuthApplicationStore = defineStore(
  'oauth-application',
  () => {
    return { application: ref<OAuthApplication | undefined>(undefined) };
  },
  { persist: true },
);
