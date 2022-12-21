<template>
  <div>
    <!-- ADD BACKGROUND IMAGE AS A <img> ELEMENT -->
    <div class="main-container">
      <div class="main-intro">
        <h2 
        
        class="main-intro-header">
        Kitsune
        <!-- {{ result.instance.domain }} -->
        </h2>
        <p class="main-intro-description">
          Lorem Ipsum is simply dummy text of the printing and typesetting
          industry. Lorem Ipsum has been the industry's standard dummy text ever
          since the 1500s, when an unknown printer took a galley of type and
          scrambled it to make a type specimen book. It has survived not only
          five centuries, but also the leap into electronic typesetting,
          remaining essentially unchanged.
        </p>
        <router-link class="main-intro-more" to="/about">Learn more</router-link>
      </div>
      <Forms />
    </div>
    <Footer />
  </div>
</template>

<script setup lang="ts">
  import { useQuery } from '@vue/apollo-composable';
  import gql from 'graphql-tag';
  import Footer from '../components/Footer.vue';
  import Forms from '../components/Forms.vue';

  const { result } = useQuery(gql`
    query getInstanceInfo {
      instance {
        domain
      }
    }
  `);
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;
  .main {
    &-container {
      display: flex;
      align-items: center;
      height: 88vh;
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

      &-header {
        font-size: 42px;
        font-weight: bold;
        color: $shade2light;
      }

      &-description,
      &-more {
        font-size: 18px;
        line-height: 143%;
        margin: 10px 0;
      }
    }
  }
</style>
