<template>
  <nav class="nav-bar">
    <div class="nav-bar-links">
      <template v-for="(details, route) in links" :key="route">
        <NavBarLink :to="route" :icon="details.icon" :detail="details.detail" />
      </template>
    </div>

    <div class="nav-bar-profile">
      <div class="nav-bar-element profile-menu-button">
        <img :src="DEFAULT_PROFILE_PICTURE_URL" />
      </div>

      <div class="nav-bar-element">
        <font-awesome-icon
          class="icon create-status"
          icon="fa-pen-to-square fa-solid"
          @click="showPostModal = true"
        />
      </div>
    </div>
  </nav>

  <NewPostModal v-model="showPostModal" />
</template>

<script setup lang="ts">
  import { defineAsyncComponent, ref } from 'vue';

  import { DEFAULT_PROFILE_PICTURE_URL } from '../consts';
  import NavBarLink from './NavBarLink.vue';

  type RouteInfo = {
    icon: string;
    detail: string;
  };

  const links: Record<string, RouteInfo> = {
    '/timeline/home': {
      icon: 'fa-house fa-solid',
      detail: 'Home',
    },
    '/notifications': {
      icon: 'fa-bell fa-solid',
      detail: 'Notification',
    },
    '/messages': {
      icon: 'fa-envelope fa-solid',
      detail: 'Messages',
    },
    '/timeline/local': {
      icon: 'fa-users fa-solid',
      detail: 'Local',
    },
    '/timeline/federated': {
      icon: 'fa-globe-europe fa-solid',
      detail: 'Federated',
    },
  };

  const NewPostModal = defineAsyncComponent(
    () => import('./modal/NewPostModal.vue'),
  );
  const showPostModal = ref(false);
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;
  @use '../styles/mixins' as *;

  .nav-bar {
    display: flex;
    position: fixed;
    top: 0;
    right: 0;
    left: 0;
    justify-content: space-between;
    align-items: center;
    z-index: 999;
    margin-bottom: 100px;
    background-color: $dark2;
    padding: 0 25px;
    padding-top: 5px;

    @include only-on-mobile {
      padding: 0;
      padding-top: 5px;

      & .detail {
        display: none;
      }

      & .icon {
        margin-right: 0px;
      }
    }

    &-profile {
      display: flex;
      gap: 10px;

      .create-status {
        cursor: pointer;
        height: 25px;
      }

      .profile-menu-button {
        display: flex;
        align-items: center;
        border-radius: 4px;

        img {
          height: 30px;
        }
      }
    }
  }
</style>
