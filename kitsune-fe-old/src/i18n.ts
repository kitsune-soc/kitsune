import { FluentBundle } from '@fluent/bundle';

import { createFluentVue } from 'fluent-vue';

const enBundle = new FluentBundle('en');
const enCyberpunkBundle = new FluentBundle('en-cyberpunk');

const fluent = createFluentVue({
  bundles: [enBundle],
});

const availableLanguages = {
  en: [enBundle],
  'en-cyberpunk': [enCyberpunkBundle, enBundle],
};

export { fluent, availableLanguages };
