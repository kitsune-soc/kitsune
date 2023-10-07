<template>
  <div class="post">
    <strong><span v-html="subject" /></strong>
    <p>
      <span v-html="content" />
    </p>
  </div>
</template>

<script lang="ts" setup>
  type Props = {
    subject: string;
    content: string;
  };

  defineProps<Props>();
</script>

<style lang="scss" scoped>
  .post {
    border: 1px solid white;
    padding: 10px;
  }
</style>
