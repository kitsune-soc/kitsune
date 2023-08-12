<template>
  <div class="forms">
    <FormKit
      type="form"
      name="login-form"
      submit-label="Login"
      @submit="login"
    />

    <FormKit
      v-if="instanceData?.registrationsOpen"
      type="form"
      submit-label="Register"
      @submit="register"
    >
      <FormKit
        type="text"
        name="username"
        validation="required"
        label="Username"
        placeholder="aumetra"
      />

      <FormKit
        type="email"
        name="email"
        validation="email|required"
        label="Email address"
        placeholder="aumetra@citadel-station.example"
      />

      <FormKit
        type="password"
        name="password"
        validation="required|zxcvbn"
        validation-visibility="dirty"
        label="Password"
      />

      <FormKit
        type="password"
        name="password_confirm"
        validation="required|confirm"
        label="Confirm password"
        validation-label="Password confirmation"
      />
    </FormKit>

    <BaseModal :closed="!modalData.show" :title="modalData.title">
      <!-- This is returned from the backend and created from an error type, and only "enhanced" with HTML newlines by us -->
      <!-- eslint-disable-next-line vue/no-v-html -->
      <span v-html="modalData.content" />

      <p>
        <button @click="modalData.show = false">Close</button>
      </p>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
  import { useMutation } from '@vue/apollo-composable';

  import gql from 'graphql-tag';
  import { reactive } from 'vue';

  import { useInstanceInfo } from '../graphql/instance-info';
  import { authorizationUrl } from '../lib/oauth2';
  import BaseModal from './BaseModal.vue';

  const modalData = reactive({
    show: false,
    title: '',
    content: '',
  });

  type RegisterData = {
    username: string;
    email: string;
    password: string;
  };

  const {
    mutate: registerUser,
    onDone: onRegisterDone,
    onError: onRegisterError,
  } = useMutation(gql`
    mutation registerUser(
      $username: String!
      $email: String!
      $password: String!
    ) {
      registerUser(username: $username, email: $email, password: $password) {
        id
      }
    }
  `);

  onRegisterDone(() => {
    modalData.title = 'Register';
    modalData.content = 'Registered successful!';
    modalData.show = true;
  });

  onRegisterError((err) => {
    modalData.title = 'Register';
    modalData.content = `Registration failed: ${err.message}`.replaceAll(
      '\n',
      '<br />',
    );
    modalData.show = true;
  });

  const instanceData = useInstanceInfo();

  async function login(): Promise<void> {
    const url = await authorizationUrl();
    window.location.href = url;
  }

  async function register(registerData: RegisterData): Promise<void> {
    await registerUser({
      username: registerData.username,
      email: registerData.email,
      password: registerData.password,
    });
  }
</script>

<style lang="scss">
  @use '../styles/colours' as *;

  form[name='login-form'] {
    align-items: center;
  }

  .formkit-form {
    background-color: $dark2;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: 90%;
    margin: 0 auto;
    padding: 3vh;
    border-radius: 5px;
    border: 0.2px solid $shade1dark;
  }

  .formkit-wrapper {
    margin: 10px auto;
  }

  .formkit-input[type='submit'] {
    border: 0;
    background-color: $shade1dark;
    border-radius: 5px;
    padding: 10px;
    font-size: 16px;
    width: 100px;
    cursor: pointer;
    transition: 0.5s;

    &:hover {
      background-color: $shade2dark;
    }
  }

  .formkit-input:not([type='submit']) {
    width: 100%;
    border: 0.5px solid $shade1dark;
    background-color: $dark1;
    border-radius: 2px;
    font-size: 20px;
    color: white;
    padding: 5px;
  }

  .forms {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-end;
    width: 40%;
    padding: 1vw;
    gap: 20px;

    @media only screen and (max-width: 1367px) {
      align-items: center;
      width: 45%;
    }

    @media only screen and (max-width: 1023px) {
      width: 66%;
    }
  }

  .formkit-messages {
    color: red;
    list-style: none;
    padding-left: 0;
  }

  .formkit-label {
    text-transform: uppercase;
    margin-bottom: 5px;
  }
</style>
