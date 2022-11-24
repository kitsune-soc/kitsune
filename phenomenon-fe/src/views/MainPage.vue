<template>
  <div class="fuck">
    <!-- ADD BACKGROUND IMAGE AS A <img> ELEMENT -->
    <div class="main-container">
      <div class="main-intro">
        <h2 class="main-intro-header">Phenomenon</h2>
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
        <form class="main-login">
          <div class="field-group">
            <label class="label" for="username">Username</label><br />
            <input
              class="field"
              type="text"
              id="username"
              name="username"
              value=""
            /><br />
            <label class="label" for="password">Password</label><br />
            <input
              class="field"
              type="password"
              id="password"
              name="password"
              value=""
            />
          </div>
          <input class="formButton" type="submit" value="Login" />
        </form>
        <form class="main-register">
          <div class="field-group">
            <label class="label" for="username">Username</label><br />
            <input
              class="field"
              type="text"
              id="username"
              name="username"
              value=""
            /><br />
            <label class="label" for="email">Email</label><br />
            <input
              class="field"
              type="email"
              id="email"
              name="email"
              value=""
            /><br />
            <label class="label" for="password">Password</label><br />
            <input
              class="field"
              type="password"
              id="password"
              name="password"
              value=""
            /><br />
            <label class="label" for="confirm-password">Confirm Password</label
            ><br />
            <input
              class="field"
              type="password"
              id="confirm-password"
              name="confirm-password"
              value=""
            />
          </div>
          <input class="formButton" type="submit" value="Register" />
        </form>
      </div>
    </div>
    <footer>
      <div class="main-footer">
        <span>Phenomenon v0.1</span>
        <a href="/">Source code</a>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
  @use '../styles/colours' as *;
  .fuck {
    // background-image: url('/assets/BG.webp');
    // background-size: cover;
    // backdrop-filter: blur(5px) saturate(1.4);
    // background-position: center;
    // background-repeat: no-repeat;
  }
  .main {
    &-container {
      display: flex;
      align-items: center;
      height: 82vh;
      width: 78vw;
      margin: 0 auto;
      gap: 30px;
    }

    &-intro {
      display: flex;
      flex-direction: column;
      justify-content: center;
      width: 50%;
      height: 90%;
      padding: 1vw;

      &-header {
        font-size: 42px;
        font-weight: bold;
        color: $shade2light;
      }

      &-description,
      &-more {
        font-size: 20px;
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
    }

    &-login,
    &-register {
      background-color: $dark2;
      width: 60%;
      padding: 5vh 2vw;
    }

    &-footer {
      display: flex;
      justify-content: center;
      align-items: flex-end;
      padding: 10px 0;
      gap: 25px;
    }
  }

  .label {
    text-transform: uppercase;
  }

  .field-group {
    margin-bottom: 10px;
  }

  .field {
    width: 280px;
    border: 0.5px solid $shade1dark;
    background-color: $dark1;
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
