<template>
  <div v-for="shit in data?.homeTimeline.nodes">
    Post ID: {{ shit?.id }} Account ID: {{ shit?.account?.id }}
  </div>
</template>

<script lang="ts" setup>
  import { watch } from 'vue';

  import { getHome } from '../../graphql/timeline';

  const { data } = getHome() as any;

  watch(
    data,
    (neww, old) => {
      console.log(old);
      console.log(neww);
    },
    { deep: true },
  );
</script>
