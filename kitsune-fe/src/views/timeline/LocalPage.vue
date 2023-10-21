<template>
  <BaseTimeline :posts="posts" :load-more="loadMore" />
</template>

<script lang="ts" setup>
  import { ref, watch } from 'vue';

  import BaseTimeline from '../../components/BaseTimeline.vue';
  import { Post } from '../../components/Post.vue';
  import { MAX_UUID } from '../../consts';
  import { getPublic } from '../../graphql/timeline';

  const posts = ref<Post[]>([]);
  const lastPostId = ref<string>(MAX_UUID);

  const localTimelineQuery = getPublic(lastPostId, true);
  watch(localTimelineQuery, (newTimelineQuery) => {
    posts.value = newTimelineQuery?.publicTimeline.nodes ?? [];
  });

  async function loadMore(): Promise<void> {
    lastPostId.value = posts.value[posts.value.length - 1].id;
  }
</script>
