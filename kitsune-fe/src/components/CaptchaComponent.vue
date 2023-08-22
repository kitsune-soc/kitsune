<template>
  <FormKit
    v-model="captchaState.token"
    type="hidden"
    name="captchaToken"
    validation="required"
  />
  <vue-hcaptcha
    v-if="backend == CaptchaBackend.HCaptcha"
    :sitekey="sitekey"
    @verify="onVerify"
    @expired="onExpire"
    @error="onError"
    @challenge-expired="onExpire"
  >
  </vue-hcaptcha>
  <div v-if="backend === CaptchaBackend.MCaptcha">
    <div id="mcaptcha__widget-container"></div>
  </div>
</template>

<script setup lang="ts">
  import { defineAsyncComponent, reactive, onMounted } from 'vue';

  import { CaptchaBackend } from '../graphql/types/graphql';

  const VueHcaptcha = defineAsyncComponent(
    () => import('@hcaptcha/vue3-hcaptcha'),
  );

  const props = defineProps<{
    backend: CaptchaBackend;
    sitekey: string;
  }>();

  const captchaState = reactive({
    verified: false,
    expired: false,
    token: '',
    error: '',
  });

  function onVerify(tokenStr: string) {
    captchaState.verified = true;
    captchaState.token = tokenStr;
  }

  function onExpire() {
    captchaState.verified = false;
    captchaState.token = '';
    captchaState.expired = true;
  }

  function onError(err: string) {
    captchaState.token = '';
    captchaState.error = err;
  }

  onMounted(async () => {
    if (props.backend == CaptchaBackend.MCaptcha) {
      const config = {
        widgetLink: new URL(props.sitekey),
      };
      const mCaptchaGlue = await import('@mcaptcha/vanilla-glue');
      // this is the only way to capture mCaptcha token
      window.addEventListener('message', (e) => {
        captchaState.token = e.data.token;
        captchaState.verified = true;
      });
    }
  });
</script>

<style lang="scss">
  @use '../styles/colours' as *;

  #mcaptcha__widget-container {
    height: 80px;
    border: 1px solid $shade1dark;
  }
</style>
