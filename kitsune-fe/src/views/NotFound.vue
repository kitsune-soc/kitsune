<template>
  <div class="container">
    <GlitchedElement>
      <h1>404</h1>
    </GlitchedElement>

    <p>
      {{ $t('messages.notFoundPage.main') }}
    </p>

    <strong class="about-link">
      <router-link to="/">
        {{ $t('messages.notFoundPage.link') }}
      </router-link>
    </strong>
  </div>
</template>

<script lang="ts" setup>
  import { GlitchedElement } from 'vue-powerglitch';
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .container {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    height: 75vh;
  }
</style>
