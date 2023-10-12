<template>
  <div class="post-view">
    <Post
      v-if="postData"
      :id="postData.id"
      :subject="postData.subject"
      :content="postData.content"
      :account="postData.account"
      :attachments="postData.attachments"
    />
  </div>
</template>

<script lang="ts" setup>
  import { useRoute } from 'vue-router';

  import Post from '../components/Post.vue';
  import { getPostById } from '../graphql/post';

  const route = useRoute();
  const postData = getPostById(<string>route.params.id);
</script>

<style lang="scss" scoped>
  .post-view {
    margin: auto;
    max-width: 100ch;
  }
</style>
