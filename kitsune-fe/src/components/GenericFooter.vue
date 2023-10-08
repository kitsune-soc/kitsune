<template>
  <footer>
    <div class="footer">
      <LanguageSelector />

      <span>Kitsune v{{ instanceData?.version }}</span>

      <a target="_blank" href="https://github.com/kitsune-soc/kitsune">
        Source code
      </a>
    </div>
  </footer>
</template>

<script setup lang="ts">
  import { useInstanceInfo } from '../graphql/instance-info';
  import LanguageSelector from './LanguageSelector.vue';

  const instanceData = useInstanceInfo();
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .footer {
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding: 10px 0;
    gap: 25px;
    margin: 25px 0;
  }
</style>
