import { usePreferredLanguages } from '@vueuse/core';

import { computed, watch } from 'vue';
import { createI18n, useI18n } from 'vue-i18n';

import en from './en';
import enCyberpunk from './en-cyberpunk';

const messages = {
  en,
  'en-cyberpunk': enCyberpunk,
};

const preferredLanguages = usePreferredLanguages();
const preferredLanguage = computed(
  () => (preferredLanguages.value[0] ?? 'en').split('-')[0],
);

watch(preferredLanguage, (newPreferredLanguage) => {
  const i18n = useI18n();
  if (newPreferredLanguage in i18n.availableLocales) {
    i18n.locale.value = newPreferredLanguage;
  }
});

const i18n = createI18n({
  locale: preferredLanguage.value,
  fallbackLocale: 'en',
  messages,
});

export { i18n };
