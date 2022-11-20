<script setup lang="ts">
  // This starter template is using Vue 3 <script setup> SFCs
  // Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
  import HelloWorld from './components/HelloWorld.vue';
</script>

<template>
  <router-link to="/">Go to Home</router-link>
  <router-link to="/about">Go to About</router-link>
  <router-view></router-view>
</template>

<style scoped></style>
