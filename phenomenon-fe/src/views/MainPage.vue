<template>
  <div>
    <!-- ADD BACKGROUND IMAGE AS A <img> ELEMENT -->
    <div class="main-container">
      <div class="main-intro">
        <h2 v-if="result" class="main-intro-header">
          {{ result.instance.domain }}
        </h2>
        <p class="main-intro-description">
          Lorem Ipsum is simply dummy text of the printing and typesetting
          industry. Lorem Ipsum has been the industry's standard dummy text ever
          since the 1500s, when an unknown printer took a galley of type and
          scrambled it to make a type specimen book. It has survived not only
          five centuries, but also the leap into electronic typesetting,
          remaining essentially unchanged.
        </p>
        <router-link class="main-intro-more" to="/about"
          >Learn more</router-link
        >
      </div>
      <div class="main-forms">
        <form class="main-login" @submit="login">
          <div class="field-group">
            <label class="label" for="username">Username</label><br />
            <input
              v-model="loginData.username"
              class="field"
              type="text"
              name="username"
            /><br />
            <label class="label" for="password">Password</label><br />
            <input
              v-model="loginData.password"
              class="field"
              type="password"
              name="password"
            />
          </div>
          <input class="formButton" type="submit" value="Login" />
        </form>
        <form class="main-register" @submit="register">
          <div class="field-group">
            <label class="label" for="username">Username</label><br />
            <input
              v-model="registerData.username"
              class="field"
              type="text"
              name="username"
            /><br />
            <label class="label" for="email">Email</label><br />
            <input
              v-model="registerData.email"
              class="field"
              type="email"
              name="email"
            /><br />
            <label class="label" for="password">Password</label><br />
            <input
              v-model="registerData.password"
              class="field"
              type="password"
              name="password"
            /><br />
            <label class="label" for="confirm-password">Confirm Password</label
            ><br />
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
    </div>
    <Footer />
  </div>
</template>

<script setup lang="ts">
  import { useMutation, useQuery } from '@vue/apollo-composable';
  import gql from 'graphql-tag';
  import { reactive } from 'vue';
  import Footer from '../components/Footer.vue';

  const { result } = useQuery(gql`
    query getInstanceInfo {
      instance {
        domain
      }
    }
  `);

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

  onDone(() => {
    // TODO: Show to user
    console.log('Registered successfully');
  });

  onError((err) => {
    // TODO: Show to user
    console.log(`Registration failed: ${err}`);
  });

  const loginData = reactive({
    username: '',
    password: '',
  });

  const registerData = reactive({
    username: '',
    email: '',
    password: '',
    passwordConfirm: '',
  });

  const login = (event: Event) => {
    event.preventDefault();

    // TODO: Start login process
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
  .main {
    &-container {
      display: flex;
      align-items: center;
      height: 82vh;
      width: 78vw;
      margin: 0 auto;
      gap: 30px;

      @media only screen and (max-width: 1023px) {
        flex-direction: column;
        height: auto;
        width: 88vw;
        justify-content: center;
        padding: 2vh 0;
      }
    }

    &-intro {
      display: flex;
      flex-direction: column;
      justify-content: center;
      width: 50%;
      height: 90%;
      padding: 1vw;

      @media only screen and (max-width: 1023px) {
        width: 75%;
        margin: 0 auto;
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
      }
    }

    &-forms {
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: flex-end;
      width: 50%;
      height: 90%;
      padding: 0.5vh 1vw;
      gap: 30px;

      @media only screen and (max-width: 1023px) {
        align-items: center;
        width: 75%;
      }
    }

    &-login,
    &-register {
      background-color: $dark2;
      width: 60%;
      padding: 5vh 2vw;
    }
  }

  .label {
    text-transform: uppercase;
  }

  .field-group {
    margin-bottom: 10px;
  }

  .field {
    width: 95%;
    border: 0.5px solid $shade1dark;
    background-color: $dark1;
    margin: 0 auto;
    margin-bottom: 8px;
    border-radius: 2px;
    font-size: 20px;
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
