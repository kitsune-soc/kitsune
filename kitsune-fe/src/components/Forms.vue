<template>
  <div class="forms-forms">
    <form class="forms-login" @submit="login">
      <input class="formButton" type="submit" value="Login" />
    </form>
    <form class="forms-register" @submit="register">
      <div class="field-group">
        <label class="label" for="username">Username</label>
        <input
          v-model="registerData.username"
          class="field"
          type="text"
          name="username"
        />
        <label class="label" for="email">Email</label>
        <input
          v-model="registerData.email"
          class="field"
          type="email"
          name="email"
        />
        <label class="label" for="password">Password</label>
        <input
          v-model="registerData.password"
          class="field"
          type="password"
          name="password"
        />
        <label class="label" for="confirm-password">Confirm Password</label>
        <input
          v-model="registerData.passwordConfirm"
          class="field"
          type="password"
          name="confirm-password"
        />
      </div>
      <input class="formButton" type="submit" value="Register" />
    </form>
  </div>
</template>

<script setup lang="ts">
  import { useMutation } from '@vue/apollo-composable';
  import gql from 'graphql-tag';
  import { reactive } from 'vue';
  import { useInstanceInfo } from '../graphql/instance-info';

  const {
    mutate: registerUser,
    onDone,
    onError,
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

  const instanceData = useInstanceInfo();

  onDone(() => {
    // TODO: Use actual modal
    alert('Registered successfully');
  });

  onError((err) => {
    // TODO: Use actual modal
    alert(`Registration failed: ${err}`);
  });

  const registerData = reactive({
    username: '',
    email: '',
    password: '',
    passwordConfirm: '',
  });

  const login = (event: Event) => {
    event.preventDefault();

    // TODO: Actually log in and not redirect to Elk
    window.location.href = `https://elk.zone/${instanceData.value?.instance.domain}/public/local`;
  };

  const register = (event: Event) => {
    event.preventDefault();

    if (
      registerData.username.trim() === '' ||
      registerData.email === '' ||
      registerData.password === ''
    ) {
      return;
    }

    if (registerData.password !== registerData.passwordConfirm) {
      return;
    }

    registerUser({
      username: registerData.username,
      email: registerData.email,
      password: registerData.password,
    });
  };
</script>

<style scoped lang="scss">
  @use '../styles/colours' as *;

  .forms {
    &-forms {
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

    &-login,
    &-register {
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

    &-login {
      align-items: center;
    }
  }

  .label {
    text-transform: uppercase;
    margin-bottom: 5px;
  }

  .field-group {
    margin-bottom: 10px;
    display: flex;
    flex-direction: column;
  }

  .field {
    width: 100%;
    border: 0.5px solid $shade1dark;
    background-color: $dark1;
    margin: 10px auto;
    border-radius: 2px;
    font-size: 20px;
    color: #fff
  }

  .formButton {
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
</style>
