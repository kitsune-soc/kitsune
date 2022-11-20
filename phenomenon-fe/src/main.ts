import { createApp, h, provide } from 'vue';
import './style.scss';
import App from './App.vue';
import { router } from './router';

import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
} from '@apollo/client/core';
import { DefaultApolloClient } from '@vue/apollo-composable';

const httpLink = createHttpLink({
  uri: '/graphql',
});

const cache = new InMemoryCache();
const apolloClient = new ApolloClient({
  link: httpLink,
  cache,
});

createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient);
  },
  render: () => h(App),
})
  .use(router)
  .mount('#app');
