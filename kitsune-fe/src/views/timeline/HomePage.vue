<template>
  <BaseTimeline :posts="processedData" />
</template>

<script lang="ts" setup>
  import { computed } from 'vue';

  import BaseTimeline from '../../components/BaseTimeline.vue';
  import { getHome } from '../../graphql/timeline';

  const data = getHome();
  const processedData = computed(() =>
    data.value ? data.value.homeTimeline.nodes : [],
  );
</script>
