<template>
  <nav class="nav-bar">
    <div class="nav-bar-links">
      <div>
        <!-- LOGO -->
      </div>
      <router-link class="nav-bar-element" to="/home">
        <font-awesome-icon class="icon" icon="fa-house fa-solid" />
        Home
      </router-link>
      <router-link class="nav-bar-element" to="/notifications">
        <font-awesome-icon class="icon" icon="fa-bell fa-solid" />
        Notifications
      </router-link>
      <router-link class="nav-bar-element" to="/messages">
        <font-awesome-icon class="icon" icon="fa-envelope fa-solid" />
        Messages
      </router-link>
      <router-link class="nav-bar-element" to="/local">
        <font-awesome-icon class="icon" icon="fa-users fa-solid" />
        Local
      </router-link>
      <router-link class="nav-bar-element" to="/federated">
        <font-awesome-icon class="icon" icon="fa-globe-europe fa-solid" />
        Federated
      </router-link>
    </div>
    <div class="nav-bar-profile">
      <div class="nav-bar-element profile-menu-button">
        <img src="https://avatarfiles.alphacoders.com/267/thumb-267407.png" />
      </div>
      <div class="nav-bar-element">
        <font-awesome-icon class="icon create-status" icon="fa-pen-to-square fa-solid" />
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .nav-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background-color: $dark2;
    padding: 0 50px;
    padding-top: 5px;
    margin-bottom: 100px;
    display: flex;
    justify-content: space-between;
    align-items: center;

    &-element {
      display: inline-block;
      padding: 15px;
      color: $shade1dark;
      cursor: pointer;

      &:hover {
        color: white;
      }

      .icon {
        margin-right: 7px;
      }

      &.router-link-active {
        color: $shade2light;
        border-bottom: 4px solid;
        border-image-slice: 1;
        border-image-source: linear-gradient(
          to left,
          $shade2light,
          $shade2dark
        );
      }
    }

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
