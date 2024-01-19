import { FluentBundle } from '@fluent/bundle';

import { createFluentVue } from 'fluent-vue';

import enCyberpunkMessages from './locales/en-cyberpunk.ftl';
import enMessages from './locales/en.ftl';

const enBundle = new FluentBundle('en');
enBundle.addResource(enMessages);

const enCyberpunkBundle = new FluentBundle('en-cyberpunk');
enCyberpunkBundle.addResource(enCyberpunkMessages);

const fluent = createFluentVue({
  bundles: [enBundle],
});

const availableLanguages = {
  en: [enBundle],
  'en-cyberpunk': [enCyberpunkBundle, enBundle],
};

export { fluent, availableLanguages };
