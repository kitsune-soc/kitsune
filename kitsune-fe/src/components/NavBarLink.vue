<template>
  <router-link :class="['nav-bar-link', $props.class]" :to="to">
    <font-awesome-icon class="icon" :icon="icon" />
    <span class="detail">{{ detail }}</span>
  </router-link>
</template>

<style lang="scss" scoped>
  @use '../styles/colours' as *;

  .nav-bar-link {
    display: inline-block;
    padding: 15px;
    color: $shade1dark;
    cursor: pointer;

    border-bottom: 4px solid;
    border-color: transparent;

    &:hover {
      color: white;
    }

    @media only screen and (max-width: 850px) {
      & .detail {
        display: none;
      }
    }

    & .icon {
      margin-right: 7px;
    }

    &.router-link-active {
      color: $shade2light;
      border-image-slice: 1;
      border-image-source: linear-gradient(to left, $shade2light, $shade2dark);
    }
  }
</style>

<script lang="ts" setup>
  defineProps<{
    class?: string;
    to: string;
    icon: string;
    detail: string;
  }>();
</script>
