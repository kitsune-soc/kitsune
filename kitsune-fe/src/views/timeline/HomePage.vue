<template>
  <fieldset class="home-timeline">
    <legend>INCOMING TRANSMISSIONS</legend>
    <div class="post-container" v-for="shit in data?.homeTimeline.nodes">
      <Post :subject="shit?.subject" :content="shit?.content" />
    </div>
  </fieldset>
</template>

<script lang="ts" setup>
  import Post from '../../components/Post.vue';
  import { getHome } from '../../graphql/timeline';

  const { data } = getHome() as any;
</script>

<style lang="scss" scoped>
  @use '../../styles/colours' as *;

  .home-timeline {
    margin: auto;
    max-width: 90ch;
    border-color: grey;
  }

  .post-container {
    margin-bottom: 15px;
  }
</style>
