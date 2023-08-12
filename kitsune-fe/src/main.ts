import {
  plugin as FormkitPlugin,
  defaultConfig as defaultFormkitConfig,
} from '@formkit/vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import urql from '@urql/vue';

import { createPinia } from 'pinia';
import piniaPluginPersistedState from 'pinia-plugin-persistedstate';
import { createApp } from 'vue';

import App from './App.vue';
import './icons';
import { router } from './router';
import './styles/root.scss';
import { urqlClient } from './urql';

const pinia = createPinia().use(piniaPluginPersistedState);

createApp(App)
  .component('font-awesome-icon', FontAwesomeIcon)
  .use(FormkitPlugin, defaultFormkitConfig)
  .use(pinia)
  .use(router)
  .use(urql, urqlClient)
  .mount('#app');
