<template>
  <article class="post">
    <div class="account-info">
      <a :href="account.url">
        <img
          class="account-info-profile-picture"
          :src="profilePictureUrl"
          :alt="`${account.username}'s profile picture`"
        />
        <div class="account-info-names">
          <strong class="account-info-names-displayname">
            {{ account.displayName ? account.displayName : account.username }}
          </strong>
          <span class="account-info-names-username">
            @{{ account.username }}
          </span>
        </div>
      </a>
    </div>
    <p v-if="subject">
      <strong><span v-html="subject" /></strong>
    </p>
    <span class="post-content" v-html="content" />
  </article>
</template>

<script lang="ts" setup>
  import { computed } from 'vue';

  import { DEFAULT_PROFILE_PICTURE_URL } from '../consts';

  export type PostAccount = {
    displayName?: string | null;
    username: string;
    avatar?: {
      url: string;
    } | null;
    url: string;
  };

  export type Post = {
    subject?: string | null;
    content: string;
    account: PostAccount;
  };

  const props = defineProps<Post>();
  const profilePictureUrl = computed(() =>
    props.account.avatar
      ? props.account.avatar.url
      : DEFAULT_PROFILE_PICTURE_URL,
  );
</script>

<style lang="scss" scoped>
  @use '../styles/colours' as *;

  .account-info {
    display: flex;

    align-items: center;
    gap: 0.5em;
    line-height: 100%;

    &-profile-picture {
      width: 4em;
    }

    &-names {
      display: flex;
      flex-direction: column;

      &-displayname {
        font-size: large;
      }
    }
  }

  .post {
    border: 1px solid white;
    border-radius: 3px;
    padding: 1em;

    background-color: $dark2;

    &-content {
      word-wrap: break-word;
      white-space: pre-wrap;
    }
  }
</style>
