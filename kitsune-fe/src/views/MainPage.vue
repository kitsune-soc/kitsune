<template>
  <div>
    <!-- ADD BACKGROUND IMAGE AS A <img> ELEMENT -->
    <div class="main-container">
      <div class="main-intro">
        <h2 class="main-intro-header">
          <svg class="main-intro-header-logo">
            <use xlink:href="/header.svg#logo" />
          </svg>
        </h2>

        <p>
          <span class="stat-highlight">
            {{ instanceInfo?.name }}
          </span>
          is home to
          <span class="stat-highlight">
            {{ instanceInfo?.userCount }}
          </span>
          users who authored
          <span class="stat-highlight">
            {{ instanceInfo?.localPostCount }}
          </span>
          posts!
        </p>

        <strong class="about-link">
          <router-link to="/about">About this instance</router-link>
        </strong>
      </div>

      <AuthForms />
    </div>

    <Footer />
  </div>
</template>

<script setup lang="ts">
  import AuthForms from '../components/AuthForms.vue';
  import Footer from '../components/Footer.vue';
  import { useInstanceInfo } from '../graphql/instance-info';

  const instanceInfo = useInstanceInfo();
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .main {
    &-container {
      display: flex;
      align-items: center;
      height: 80vh;
      width: 95vw;
      margin: 0 auto;
      padding: 0 4vw;

      @media only screen and (max-width: 1023px) {
        flex-direction: column;
        height: auto;
        justify-content: center;
        padding: 3vh 4vw;
      }
    }

    &-intro {
      display: flex;
      flex-direction: column;
      justify-content: center;
      width: 60%;
      height: auto;
      padding: 1vh 2vw;

      @media only screen and (max-width: 1367px) {
        width: 55%;
      }

      @media only screen and (max-width: 1023px) {
        width: 75%;
        margin-bottom: 4vh;
        text-align: center;
      }

      & .stat-highlight {
        color: $shade1dark;
      }

      &-header {
        font-size: 42px;
        font-weight: bold;
        color: $shade2light;

        &-logo {
          color: $shade2light;
          width: 500px;
          max-width: 100%;
        }
      }

      &-description,
      &-more {
        width: fit-content;
        font-size: 18px;
        line-height: 143%;
        margin: 10px 0;
      }
    }
  }
</style>
