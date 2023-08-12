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
import { zxcvbnRule, zxcvbnValidationMessage } from './zxcvbn';

const pinia = createPinia().use(piniaPluginPersistedState);

createApp(App)
  .component('font-awesome-icon', FontAwesomeIcon)
  .use(
    FormkitPlugin,
    defaultFormkitConfig({
      messages: {
        en: {
          validation: {
            zxcvbn: zxcvbnValidationMessage,
          },
        },
      },
      rules: {
        zxcvbn: zxcvbnRule,
      },
    }),
  )
  .use(pinia)
  .use(router)
  .use(urql, urqlClient)
  .mount('#app');
