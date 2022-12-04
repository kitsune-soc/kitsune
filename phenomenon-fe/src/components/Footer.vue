<template>
  <footer>
    <div v-if="result" class="footer">
      <span>Phenomenon v{{ result.instance.version }}</span>
      <a href="/">Source code</a>
    </div>
  </footer>
</template>

<script setup lang="ts">
  import { useQuery } from '@vue/apollo-composable';
  import gql from 'graphql-tag';

  const { result } = useQuery(gql`
    query getInstanceInfo {
      instance {
        version
      }
    }
  `);
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .footer {
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding: 10px 0;
    gap: 25px;
    margin-bottom: 25px;
  }
</style>
