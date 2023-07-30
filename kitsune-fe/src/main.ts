import { createApp, h, provide } from 'vue';
import { DefaultApolloClient } from '@vue/apollo-composable';
import './icons';
import './styles/root.scss';
import App from './App.vue';
import { apolloClient } from './apollo';
import { router } from './router';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { createPinia } from 'pinia';
import piniaPluginPersistedState from 'pinia-plugin-persistedstate';
import {
  plugin as FormkitPlugin,
  defaultConfig as defaultFormkitConfig,
} from '@formkit/vue';

const pinia = createPinia().use(piniaPluginPersistedState);

createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient);
  },
  render: () => h(App),
})
  .component('font-awesome-icon', FontAwesomeIcon)
  .use(FormkitPlugin, defaultFormkitConfig)
  .use(pinia)
  .use(router)
  .mount('#app');
