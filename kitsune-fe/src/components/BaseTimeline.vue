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
          item: PostType;
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
          <Post
            :account="item.account"
            :subject="item.subject"
            :content="item.content"
          />
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

  import Post, { Post as PostType } from './Post.vue';

  defineProps<{ posts: PostType[] }>();

  const scroller = ref<HTMLElement>();
  useInfiniteScroll(
    scroller,
    () => {
      console.log('hmm');
    },
    { distance: 3 },
  );
</script>

<style lang="scss" scoped>
  .timeline {
    margin: auto;
    border-color: grey;

    max-height: 80vh;
    max-width: 100ch;
    overflow-y: scroll;
  }

  .post-container * {
    margin-bottom: 15px;
  }

  .scroller {
    height: 100%;
  }
</style>
