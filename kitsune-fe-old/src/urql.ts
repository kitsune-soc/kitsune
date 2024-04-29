import {
  Directive,
  cacheExchange as createCacheExchange,
} from '@urql/exchange-graphcache';
import { relayPagination } from '@urql/exchange-graphcache/extras';
import { Client, Exchange, fetchExchange, mapExchange } from '@urql/vue';

import { merge } from 'lodash';

import { BACKEND_PREFIX } from './consts';
import { useAuthStore } from './store/auth';

const authExchange: Exchange = mapExchange({
  async onOperation(operation) {
    const authStore = useAuthStore();

    if (authStore.isAuthenticated()) {
      operation.context.fetchOptions = merge(
        {
          headers: {
            Authorization: `Bearer ${await authStore.accessToken()}`,
          },
        },
        operation.context.fetchOptions,
      );
    }

    return operation;
  },
});

const cacheExchange: Exchange = createCacheExchange({
  directives: {
    relayPagination: <Directive>relayPagination,
  },
});

export const urqlClient = new Client({
  url: `${BACKEND_PREFIX}/graphql`,
  exchanges: [authExchange, cacheExchange, fetchExchange],
});
