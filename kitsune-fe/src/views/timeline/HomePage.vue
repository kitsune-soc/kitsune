<template>
  <BaseTimeline :posts="posts" :load-more="loadMore" />
</template>

<script lang="ts" setup>
  import { ref, watch } from 'vue';

  import BaseTimeline from '../../components/BaseTimeline.vue';
  import { Post } from '../../components/Post.vue';
  import { MAX_UUID } from '../../consts';
  import { getHome } from '../../graphql/timeline';

  const posts = ref<Post[]>([]);
  const lastPostId = ref<string>(MAX_UUID);

  const homeTimelineQuery = getHome(lastPostId);
  watch(homeTimelineQuery, (newTimelineQuery) => {
    (newTimelineQuery?.homeTimeline.nodes ?? []).forEach((post) =>
      posts.value.push(post),
    );
  });

  async function loadMore(): Promise<void> {
    lastPostId.value = posts.value[posts.value.length - 1].id;
  }
</script>
