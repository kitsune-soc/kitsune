<template>
  <router-link
    :class="['nav-bar-link', $props.class]"
    :to="to"
    draggable="false"
  >
    <font-awesome-icon class="icon" :icon="icon" />
    <span v-if="detail" class="detail">{{ detail }}</span>
  </router-link>
</template>

<script lang="ts" setup>
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
    padding: 15px;
    color: $shade1dark;

    cursor: pointer;
    user-select: none;

    border-bottom: 4px solid;
    border-color: transparent;

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
      color: $shade2light;
      border-image-slice: 1;
      border-image-source: linear-gradient(to left, $shade2light, $shade2dark);
    }
  }
</style>
