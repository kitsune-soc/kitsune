import { createApp, h, provide } from 'vue';
import { DefaultApolloClient } from '@vue/apollo-composable';
import './style.scss';
import App from './App.vue';
import { apolloClient } from './apollo';
import { router } from './router';

createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient);
  },
  render: () => h(App),
})
  .use(router)
  .mount('#app');
