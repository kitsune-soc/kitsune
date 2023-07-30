<template>
  <div class="about-page">
    <h1 class="title">
      Kitsune hosted on
      <router-link to="/">
        {{ instanceData?.instance.domain }}
      </router-link>
    </h1>

    <fieldset>
      <legend>Instance description</legend>

      <span v-html="instanceData?.instance.description" />
    </fieldset>
    <Footer />
  </div>
</template>

<script setup lang="ts">
  import Footer from '../components/Footer.vue';
  import { useInstanceInfo } from '../graphql/instance-info';

  const instanceData = useInstanceInfo();
</script>

<style scoped lang="scss">
  h1 {
    font-size: 3em;
    line-height: 1.1;
  }

  .about-page {
    max-width: 80ch;
    margin: auto;

    & .title {
      text-align: center;
    }
  }
</style>
