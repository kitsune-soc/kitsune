<template>
  <router-link
    :class="['nav-bar-link', $props.class]"
    :to="to"
    draggable="false"
  >
    <GlitchedElement :options="{ playMode: 'hover' }">
      <font-awesome-icon class="icon" :icon="icon" />
      <span v-if="detail" class="detail">{{ detail }}</span>
    </GlitchedElement>
  </router-link>
</template>

<script lang="ts" setup>
  import { GlitchedElement } from 'vue-powerglitch';

  defineProps<{
    class?: string;
    to: string;
    icon: string;
    detail?: string;
  }>();
</script>

<style lang="scss" scoped>
  @use '../styles/colours' as *;
  @use '../styles/mixins' as *;

  .nav-bar-link {
    display: inline-block;

    cursor: pointer;

    border-bottom: 4px solid;
    border-color: transparent;
    padding: 15px;
    color: $shade1dark;
    user-select: none;

    &:hover {
      color: white;
    }

    & .icon {
      margin-right: 10px;
    }

    @include only-on-mobile {
      & .detail {
        display: none;
      }

      & .icon {
        margin-right: 0px;
      }
    }

    &.router-link-active {
      border-image-source: linear-gradient(to left, $shade2light, $shade2dark);
      border-image-slice: 1;
      color: $shade2light;
    }
  }
</style>
