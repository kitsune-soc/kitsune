<template>
  <NavBar v-if="authStore.isAuthenticated()" />
  <router-view></router-view>
  <GenericFooter />
</template>

<script setup lang="ts">
  import GenericFooter from './components/GenericFooter.vue';
  import NavBar from './components/NavBar.vue';
  import { useAuthStore } from './store/auth';

  const authStore = useAuthStore();
</script>

<style scoped lang="scss"></style>
