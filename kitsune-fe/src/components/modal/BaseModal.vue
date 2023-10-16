<template>
  <div v-if="modelValue" class="modal">
    <fieldset ref="modalContent" class="modal-content">
      <legend class="modal-title">
        {{ title }}
      </legend>
      <slot />
    </fieldset>
  </div>
</template>

<script lang="ts" setup>
  import { onClickOutside } from '@vueuse/core';

  import { ref } from 'vue';

  defineProps<{
    modelValue: boolean;
    title: string;
  }>();

  const emit = defineEmits<{
    (event: 'update:modelValue', modelValue: boolean): void;
  }>();

  const modalContent = ref();
  onClickOutside(modalContent, () => {
    emit('update:modelValue', false);
  });
</script>

<style lang="scss" scoped>
  @use '../../styles/colours' as *;

  .modal {
    display: flex;
    position: fixed;

    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    justify-content: center;
    align-items: center;

    z-index: 999;

    background-color: rgba(0, 0, 0, 0.75);

    &-title {
      text-transform: uppercase;
    }

    &-content {
      background-color: $dark1;
      height: fit-content;
    }
  }
</style>
