<template>
  <nav class="nav-bar">
    <router-link class="nav-bar-element" to="/home">
      <font-awesome-icon icon="fa-house fa-solid" />
      Home
    </router-link>
    <router-link class="nav-bar-element" to="/notifications">
      <font-awesome-icon icon="fa-bell fa-solid" />
      Notifications
    </router-link>
    <router-link class="nav-bar-element" to="/messages">
      <font-awesome-icon icon="fa-envelope fa-solid" />
      Messages
    </router-link>
    <router-link class="nav-bar-element" to="/local">
      <font-awesome-icon icon="fa-user fa-solid" />
      Local
    </router-link>
    <router-link class="nav-bar-element" to="/federated">
      <font-awesome-icon icon="fa-globe fa-solid" />
      Federated
    </router-link>
  </nav>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
  .nav-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: fit-content;
    padding: 20px;
    border-bottom: 1px solid;

    &-element {
      margin-right: 20px;
    }
  }
</style>
