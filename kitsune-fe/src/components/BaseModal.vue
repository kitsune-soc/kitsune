<template>
  <div v-if="!closed" class="modal">
    <fieldset class="modal-content">
      <legend>
        {{ title }}
      </legend>
      <slot />
    </fieldset>
  </div>
</template>

<script lang="ts" setup>
  defineProps<{
    closed: boolean;
    title: string;
  }>();
</script>

<style lang="scss" scoped>
  @use '../styles/colours' as *;

  .modal {
    position: absolute;
    display: flex;
    justify-content: center;
    align-items: center;

    top: 0;
    bottom: 0;
    left: 0;
    right: 0;

    background-color: rgba(0, 0, 0, 0.75);

    &-content {
      background-color: $dark1;
      height: fit-content;
    }
  }
</style>
