<template>
  <fieldset class="home-timeline">
    <legend>INCOMING TRANSMISSIONS</legend>
    <DynamicScroller :items="posts" :min-item-size="50">
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
          <!-- Load bearing litting div -->
          <!-- Without this div, the height computation is all messed up and the margin of the post gets ignored -->
          <div style="height: 1px"></div>
        </DynamicScrollerItem>
      </template>
    </DynamicScroller>
  </fieldset>
</template>

<script lang="ts" setup>
  export type Post = {
    subject?: string | null;
    content: string;
  };

  defineProps<{ posts: Post[] }>();
</script>
