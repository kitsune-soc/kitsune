<template>
  <select v-model="$i18n.locale">
    <option v-for="(lang, i) in $i18n.availableLocales" :key="i" :value="lang">
      {{ lang }}
    </option>
  </select>
</template>
