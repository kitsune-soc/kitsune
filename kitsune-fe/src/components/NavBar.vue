<template>
  <nav class="nav-bar">
    <div class="nav-bar-links">
      <template v-for="link in links">
        <NavBarLink :to="link.to" :icon="link.icon" :detail="link.detail" />
      </template>
    </div>
    <div class="nav-bar-profile">
      <div class="nav-bar-element profile-menu-button">
        <img src="https://avatarfiles.alphacoders.com/267/thumb-267407.png" />
      </div>
      <div class="nav-bar-element">
        <font-awesome-icon
          class="icon create-status"
          icon="fa-pen-to-square fa-solid"
        />
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
  import NavBarLink from './NavBarLink.vue';

  const links = [
    {
      to: '/home',
      icon: 'fa-house fa-solid',
      detail: 'Home',
    },
    {
      to: '/notifications',
      icon: 'fa-bell fa-solid',
      detail: 'Notification',
    },
    {
      to: '/messages',
      icon: 'fa-envelope fa-solid',
      detail: 'Messages',
    },
    {
      to: '/local',
      icon: 'fa-users fa-solid',
      detail: 'Local',
    },
    {
      to: '/federated',
      icon: 'fa-globe-europe fa-solid',
      detail: 'Federated',
    },
  ];
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .nav-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background-color: $dark2;
    padding: 0 25px;
    padding-top: 5px;
    margin-bottom: 100px;
    display: flex;
    justify-content: space-between;
    align-items: center;

    &-profile {
      display: flex;
      gap: 10px;

      .create-status {
        height: 25px;
      }

      .profile-menu-button {
        border-radius: 4px;
        display: flex;
        align-items: center;

        img {
          height: 30px;
        }
      }
    }
  }
</style>
