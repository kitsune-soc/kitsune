<template>
    <FormKit
      type="hidden"
      name="captchaToken"
      validation="required"
      v-model="captchaState.token"
    />
    <vue-hcaptcha
      v-if="backend == CaptchaBackend.HCaptcha"
      :sitekey="sitekey"
      @verify="onVerify"
      @expired="onExpire"
      @error="onError"
      @challenge-expired="onExpire">
    </vue-hcaptcha>
    <div v-if="backend === CaptchaBackend.MCaptcha">
      <div id="mcaptcha__widget-container"></div>
      <component is="script" type="text/javascript" src="/public/mcaptcha-glue.js" onload="mcaptcha__load();"></component>
      <component is="script" type="text/javascript" charset="utf-8">
        function mcaptcha__load() {
          let config = {
            widgetLink: new URL("{{sitekey}}"),
          };
          new mcaptchaGlue.default(config);
        }
      </component>
    </div>
</template>

<script setup lang="ts">
  import VueHcaptcha from '@hcaptcha/vue3-hcaptcha';
  import { reactive, onMounted } from 'vue';
  import { CaptchaBackend } from '../graphql/types/graphql';
  
 defineProps<{
    backend: CaptchaBackend;
    sitekey: string;
  }>();

  const captchaState = reactive({
    verified: false,
    expired: false,
    token: '',
    error: '',
  })

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

  // this is the only to capture mCaptcha token
  onMounted(() => {
    window.addEventListener("message", (e) => {
      captchaState.token = e.data.token;
      captchaState.verified = true;
    });
  });
</script>

<style lang="scss">
  @use '../styles/colours' as *;

  #mcaptcha__widget-container {
    height: 80px;
    border: 1px solid $shade1dark;
  }
</style>