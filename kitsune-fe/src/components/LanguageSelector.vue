<template>
  <select v-model="fluent.bundles.value">
    <option
      v-for="(bundles, name) in availableLanguages"
      :key="name"
      :value="bundles"
    >
      {{ name }}
    </option>
  </select>
</template>

<script setup lang="ts">
  import { useFluent } from 'fluent-vue';

  import { availableLanguages } from '../i18n';

  const fluent = useFluent();
</script>
