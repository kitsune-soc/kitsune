import { createApp, h, provide } from 'vue';
import { DefaultApolloClient } from '@vue/apollo-composable';
import '@fontsource/play';
import './icons';
import './styles/root.scss';
import App from './App.vue';
import { apolloClient } from './apollo';
import { router } from './router';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';

createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient);
  },
  render: () => h(App),
})
  .component('font-awesome-icon', FontAwesomeIcon)
  .use(router)
  .mount('#app');
