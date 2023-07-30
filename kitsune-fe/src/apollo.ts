import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
} from '@apollo/client/core';
import { BACKEND_PREFIX } from './consts';
import { provideApolloClient as vueProvideApolloClient } from '@vue/apollo-composable';

const httpLink = createHttpLink({
  uri: `${BACKEND_PREFIX}/graphql`,
});
const cache = new InMemoryCache();

export const apolloClient = new ApolloClient({
  link: httpLink,
  cache,
});
export const provideApolloClient = vueProvideApolloClient(apolloClient);
