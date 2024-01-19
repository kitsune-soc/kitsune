<template>
  <fieldset ref="scroller" class="timeline">
    <legend class="timeline-legend">
      {{ $t('messages-timeline-title') }}
    </legend>
    <DynamicScroller class="scroller" :items="posts" :min-item-size="50">
      <template
        #default="{
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
          :size-dependencies="[item.subject, item.content, item.attachments]"
          :data-index="index"
        >
          <Post
            :id="item.id"
            :account="item.account"
            :subject="item.subject"
            :content="item.content"
            :attachments="item.attachments"
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
  import { DynamicScroller, DynamicScrollerItem } from 'vue-virtual-scroller';

  import Post, { Post as PostType } from './Post.vue';

  const props = defineProps<{
    posts: PostType[];
    loadMore: () => Promise<void>;
  }>();

  const scroller = ref<HTMLElement>();
  useInfiniteScroll(scroller, async () => {
    if (props.posts.length !== 0) {
      await props.loadMore();
    }
  });
</script>

<style lang="scss" scoped>
  .timeline {
    margin: auto;
    border-color: grey;
    max-width: 100ch;

    max-height: 82vh;
    overflow-y: scroll;

    &-legend {
      text-transform: uppercase;
    }
  }

  .post-container * {
    margin-bottom: 15px;
  }

  .scroller {
    height: 100%;
  }
</style>
