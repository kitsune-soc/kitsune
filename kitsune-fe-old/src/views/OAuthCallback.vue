<template>
  <div class="oauth-callback">
    This is a page just here to execute some code to finish authentication
    <p>
      If you actually see this page for longer than maybe a second or two,
      something probably broke.
    </p>
  </div>
</template>

<script lang="ts" setup>
  import { onMounted } from 'vue';
  import { useRouter } from 'vue-router';

  import { obtainAccessToken } from '../lib/oauth2';

  type AuthorizationQuery = {
    code: string;
  };

  const router = useRouter();
  const route = router.currentRoute.value;

  onMounted(async () => {
    const query = route.query as AuthorizationQuery;
    await obtainAccessToken(query.code);
    await router.push('/');
  });
</script>

<style scoped>
  .oauth-callback {
    text-align: center;
  }
</style>
