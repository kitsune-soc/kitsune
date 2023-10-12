<template>
  <slot v-if="reduceMotion" />

  <GlitchedElement v-else :options="options">
    <slot />
  </GlitchedElement>
</template>

<script lang="ts" setup>
  import { usePreferredReducedMotion } from '@vueuse/core';

  import { computed } from 'vue';
  import { GlitchedElement } from 'vue-powerglitch';

  defineProps<{ options?: object }>();

  const preferredReducedMotion = usePreferredReducedMotion();
  const reduceMotion = computed(
    () => preferredReducedMotion.value === 'reduce',
  );
</script>
