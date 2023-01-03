import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
} from '@apollo/client/core';
import { BACKEND_PREFIX } from './consts';

const httpLink = createHttpLink({
  uri: `${BACKEND_PREFIX}/graphql`,
});
const cache = new InMemoryCache();

export const apolloClient = new ApolloClient({
  link: httpLink,
  cache,
});
