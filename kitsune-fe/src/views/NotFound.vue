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
  import GlitchedElement from '../components/GlitchedElement.vue';
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 75vh;
  }
</style>
