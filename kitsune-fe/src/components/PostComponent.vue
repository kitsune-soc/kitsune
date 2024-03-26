<template>
  <article class="post" @click="goToThreadView">
    <a class="account-info" :href="account.url">
      <img
        class="account-info-profile-picture"
        :src="profilePictureUrl"
        :alt="`${account.username}'s profile picture`"
      />

      <div class="account-info-names">
        <strong class="account-info-names-displayname">
          {{ account.displayName ?? account.username }}
        </strong>
        <span class="account-info-names-username">
          @{{ account.username }}
        </span>
      </div>
    </a>

    <p v-if="subject">
      <strong>
        <!-- Cleaned on the backend -->
        <!-- eslint-disable-next-line vue/no-v-html -->
        <span v-html="subject" />
      </strong>
    </p>

    <!-- Cleaned on the backend -->
    <!-- eslint-disable-next-line vue/no-v-html -->
    <span class="post-content" v-html="content" />

    <div class="post-attachments">
      <div
        v-for="attachment in attachments"
        :key="attachment.url"
        :title="attachment.description!"
      >
        <audio
          v-if="attachment.contentType.startsWith('audio')"
          :src="attachment.url"
          controls
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
  import { useRouter } from 'vue-router';

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
    id: string;
    subject?: string | null;
    content: string;
    account: PostAccount;
    attachments: PostAttachment[];
  };

  const props = defineProps<Post>();
  const profilePictureUrl = computed(
    () => props.account.avatar?.url ?? DEFAULT_PROFILE_PICTURE_URL,
  );
  const router = useRouter();

  function goToThreadView() {
    router.push(`/posts/${props.id}`);
  }
</script>

<style lang="scss" scoped>
  @use '../styles/colours' as *;

  .post {
    border: 1px solid white;
    border-radius: 3px;

    background-color: $dark2;
    padding: 1em;

    & .account-info {
      display: flex;
      align-items: center;
      gap: 0.5em;

      width: fit-content;
      line-height: 100%;

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
        max-height: 30ch;
      }
    }
  }
</style>
