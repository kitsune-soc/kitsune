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

        <div>
          <span class="stat-highlight">
            {{ instanceInfo?.name }}
          </span>
          {{ $t('stats-title') }}:
          <ul>
            <li>
              <span class="stat-highlight">
                {{ instanceInfo?.userCount }}
              </span>
              {{ $t('stats-user', { count: instanceInfo?.userCount ?? 0 }) }}
            </li>
            <li>
              <span class="stat-highlight">
                {{ instanceInfo?.localPostCount }}
              </span>
              {{
                $t('stats-post', { count: instanceInfo?.localPostCount ?? 0 })
              }}
            </li>
          </ul>
        </div>

        <strong class="about-link">
          <router-link to="/about">
            {{ $t('messages-mainPage-aboutInstance') }}
          </router-link>
        </strong>
      </div>

      <AuthForms />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { onMounted } from 'vue';
  import { useRouter } from 'vue-router';

  import AuthForms from '../components/AuthForms.vue';
  import { useInstanceInfo } from '../graphql/instance-info';
  import { useAuthStore } from '../store/auth';

  const authStore = useAuthStore();
  const instanceInfo = useInstanceInfo();

  onMounted(async () => {
    if (authStore.isAuthenticated()) {
      const router = useRouter();
      router.replace('/timeline/home');
    }
  });
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .main {
    &-container {
      display: flex;
      align-items: center;
      margin: 0 auto;
      padding: 0 4vw;
      width: 95vw;
      height: 80vh;

      @media only screen and (max-width: 1023px) {
        flex-direction: column;
        justify-content: center;
        padding: 3vh 4vw;
        height: auto;
      }
    }

    &-intro {
      display: flex;
      flex-direction: column;
      justify-content: center;
      padding: 1vh 2vw;
      width: 60%;
      height: auto;

      @media only screen and (max-width: 1367px) {
        width: 55%;
      }

      @media only screen and (max-width: 1023px) {
        margin-bottom: 4vh;
        width: 75%;
        text-align: center;
      }

      & .stat-highlight {
        display: inline;
        color: $shade1dark;
      }

      &-header {
        color: $shade2light;
        font-weight: bold;
        font-size: 42px;

        &-logo {
          width: 500px;
          max-width: 100%;
          color: $shade2light;
        }
      }

      &-description,
      &-more {
        margin: 10px 0;
        width: fit-content;
        font-size: 18px;
        line-height: 143%;
      }
    }
  }
</style>
