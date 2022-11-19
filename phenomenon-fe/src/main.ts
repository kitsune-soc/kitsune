import { createApp, h, provide } from 'vue';
import './style.scss';
import App from './App.vue';

import {createRouter, createWebHashHistory} from 'vue-router';

const Home = { template: '<p>Home</p>' }
const About = { template: '<div>About</div>' }

const routes = [
  { path: '/', component: Home },
  { path: '/about', component: About },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})


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

const app = createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient);
  },
  render: () => h(App),
})
app.use(router);
app.mount('#app');
