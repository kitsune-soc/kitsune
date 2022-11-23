<template>
  <nav class="nav-bar">
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

    <div class="create-status nav-bar-element right-side">
      <font-awesome-icon class="icon" icon="fa-pen-to-square fa-solid" />
    </div>
    <img
      class="nav-bar-element profile-menu-button right-side"
      src="https://avatarfiles.alphacoders.com/267/thumb-267407.png"
    />
  </nav>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
  @use '../styles/colours';

  .nav-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background-color: colours.$darkblue;
    padding: 0 50px;

    &-element {
      display: inline-block;
      padding: 15px;
      color: darkgrey;

      & .icon {
        margin-right: 7px;
      }

      &.create-status,
      &.profile-menu-button {
        cursor: pointer;
      }

      &.profile-menu-button {
        height: 30px;
        border-radius: 4px;
      }

      &.right-side {
        float: right;
      }

      &:hover {
        color: white;
      }

      &.router-link-active {
        color: colours.$magenta;
        border-bottom: 4px solid;
        border-image-slice: 1;
        border-image-source: linear-gradient(
          to left,
          colours.$pink,
          colours.$magenta
        );
      }
    }
  }
</style>
