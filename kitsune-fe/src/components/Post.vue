<template>
  <article class="post">
    <a class="account-info" :href="account.url">
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
    <p v-if="subject">
      <strong><span v-html="subject" /></strong>
    </p>
    <span class="post-content" v-html="content" />
    <div class="post-attachments">
      <div v-for="attachment in attachments" :title="attachment.description!">
        <audio
          v-if="attachment.contentType.startsWith('audio')"
          :src="attachment.url"
        />
        <video
          v-else-if="attachment.contentType.startsWith('video')"
          :src="attachment.url"
          controls
        />
        <img v-else :src="attachment.url" :alt="attachment.description!" />
      </div>
    </div>
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

  export type PostAttachment = {
    contentType: string;
    description?: string | null;
    url: string;
  };

  export type Post = {
    subject?: string | null;
    content: string;
    account: PostAccount;
    attachments: PostAttachment[];
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

  .post {
    border: 1px solid white;
    border-radius: 3px;
    padding: 1em;

    background-color: $dark2;

    & .account-info {
      display: flex;
      align-items: center;
      gap: 0.5em;
      line-height: 100%;

      width: fit-content;

      &-profile-picture {
        width: 3em;
      }

      &-names {
        display: flex;
        flex-direction: column;

        &-displayname {
          font-size: large;
        }
      }
    }

    &-content {
      word-wrap: break-word;
      white-space: pre-wrap;
    }

    &-attachments {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 0.25em;

      & * {
        width: 100%;
        max-height: 50ch;
      }
    }
  }
</style>
