<template>
  <fieldset class="timeline" ref="scroller">
    <legend>INCOMING TRANSMISSIONS</legend>
    <DynamicScroller class="scroller" :items="posts" :min-item-size="50">
      <template
        v-slot="{
          item,
          index,
          active,
        }: {
          item: Post;
          index: number;
          active: boolean;
        }"
      >
        <DynamicScrollerItem
          class="post-container"
          :item="item"
          :active="active"
          :size-dependencies="[item.subject, item.content]"
          :data-index="index"
        >
          <Post :subject="item.subject" :content="item.content" />
          <!-- Load bearing little div -->
          <!-- Without this div, the height computation is all messed up and the margin of the post gets ignored -->
          <div style="height: 1px"></div>
        </DynamicScrollerItem>
      </template>
    </DynamicScroller>
  </fieldset>
</template>

<script lang="ts" setup>
  import { useInfiniteScroll } from '@vueuse/core';

  import { ref } from 'vue';

  import Post from './Post.vue';

  export type Post = {
    subject?: string | null;
    content: string;
  };

  defineProps<{ posts: Post[] }>();

  const scroller = ref<HTMLElement>();
  useInfiniteScroll(
    scroller,
    () => {
      console.log('hmm');
    },
    { distance: 10 },
  );
</script>

<style lang="scss" scoped>
  .timeline {
    margin: 1em;
    border-color: grey;

    max-height: 80vh;
    overflow-y: scroll;
  }

  .post-container * {
    margin-bottom: 15px;
  }

  .scroller {
    height: 100%;
  }
</style>
