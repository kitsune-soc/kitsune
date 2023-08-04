import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
} from '@apollo/client/core';
import { setContext } from '@apollo/client/link/context';
import { provideApolloClient as vueProvideApolloClient } from '@vue/apollo-composable';

import merge from 'lodash/merge';

import { BACKEND_PREFIX } from './consts';
import { useAuthStore } from './store/auth';

const httpLink = createHttpLink({
  uri: `${BACKEND_PREFIX}/graphql`,
});
const cache = new InMemoryCache();

const authMiddleware = setContext(async (request, previousContext) => {
  const authStore = useAuthStore();

  if (authStore.isAuthenticated()) {
    previousContext = merge(previousContext, {
      headers: {
        authorization: `Bearer ${await authStore.accessToken()}`,
      },
    });
  }

  return previousContext;
});

export const apolloClient = new ApolloClient({
  link: authMiddleware.concat(httpLink),
  cache,
});
export const provideApolloClient = vueProvideApolloClient(apolloClient);
