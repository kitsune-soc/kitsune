<template>
  <div class="post">
    <p v-if="subject">
      <strong><span v-html="subject" /></strong>
    </p>
    <span v-html="content" />
  </div>
</template>

<script lang="ts" setup>
  export type PostAccount = {
    displayName?: string | null;
    username: string;
    avatar?: {
      url: string;
    } | null;
    url: string;
  };

  export type Post = {
    subject?: string | null;
    content: string;
    account: PostAccount;
  };

  defineProps<Post>();
</script>

<style lang="scss" scoped>
  .post {
    border: 1px solid white;
    padding: 10px;
  }
</style>
