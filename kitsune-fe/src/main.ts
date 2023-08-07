import {
  plugin as FormkitPlugin,
  defaultConfig as defaultFormkitConfig,
} from '@formkit/vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import { DefaultApolloClient } from '@vue/apollo-composable';

import { createPinia } from 'pinia';
import piniaPluginPersistedState from 'pinia-plugin-persistedstate';
import { createApp } from 'vue';

import App from './App.vue';
import { apolloClient } from './apollo';
import './icons';
import { router } from './router';
import './styles/root.scss';

const pinia = createPinia().use(piniaPluginPersistedState);

createApp(App)
  .component('font-awesome-icon', FontAwesomeIcon)
  .provide(DefaultApolloClient, apolloClient)
  .use(FormkitPlugin, defaultFormkitConfig)
  .use(pinia)
  .use(router)
  .mount('#app');
