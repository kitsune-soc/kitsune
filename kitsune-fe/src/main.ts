import {
  plugin as FormkitPlugin,
  defaultConfig as defaultFormkitConfig,
} from '@formkit/vue';
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';
import urql from '@urql/vue';

import { createPinia } from 'pinia';
import piniaPluginPersistedState from 'pinia-plugin-persistedstate';
import { createHead } from 'unhead';
import { createApp } from 'vue';
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css';

import App from './App.vue';
import { i18n } from './i18n';
import './icons';
import { router } from './router';
import './styles/root.scss';
import { urqlClient } from './urql';
import { zxcvbnRule, zxcvbnValidationMessage } from './zxcvbn';

createHead(); // We need to initialize `unhead` somewhere near the entry point, so yeah

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
  .use(i18n)
  .use(pinia)
  .use(router)
  .use(urql, urqlClient)
  .mount('#app');
