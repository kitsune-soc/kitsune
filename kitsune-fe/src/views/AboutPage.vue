<template>
  <div class="about-page">
    <h1 class="title">Kitsune hosted on {{ instanceData?.instance.domain }}</h1>

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
  .about-page {
    max-width: 80ch;
    margin: auto;

    & .title {
      text-align: center;
    }
  }
</style>
