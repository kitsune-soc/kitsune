<template>
  <select v-model="fluent.bundles">
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
  import { fluent, availableLanguages } from '../i18n';
</script>
