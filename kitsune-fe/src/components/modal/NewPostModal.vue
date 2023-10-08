<template>
  <BaseModal
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    :title="$t('messages.newPost.title')"
  >
    <BubbleMenu v-if="editor" :editor="editor" :tippy-options="tippyOptions">
      <button
        @click="editor.chain().focus().toggleBold().run()"
        :class="{ 'is-active': editor.isActive('bold') }"
      >
        Bold
      </button>

      <button
        @click="editor.chain().focus().toggleCodeBlock().run()"
        :class="{ 'is-active': editor.isActive('codeBlock') }"
      >
        Code block
      </button>
    </BubbleMenu>

    <FloatingMenu v-if="editor" :editor="editor" :tippy-options="tippyOptions">
      <button
        :class="{ 'is-active': editor.isActive('heading', { level: 1 }) }"
        @click="editor.chain().focus().toggleHeading({ level: 1 }).run()"
      >
        H1
      </button>
      <button
        :class="{ 'is-active': editor.isActive('heading', { level: 2 }) }"
        @click="editor.chain().focus().toggleHeading({ level: 2 }).run()"
      >
        H2
      </button>

      <button
        :class="{ 'is-active': editor.isActive('bulletList') }"
        @click="editor.chain().focus().toggleBulletList().run()"
      >
        Toggle list
      </button>
    </FloatingMenu>

    <EditorContent class="editor" :editor="editor" />

    <div class="controls">
      <div class="controls-modifiers">lmao</div>

      <button class="controls-post-button">Post!</button>
    </div>
  </BaseModal>
</template>

<script lang="ts" setup>
  import StarterKit from '@tiptap/starter-kit';
  import {
    useEditor,
    BubbleMenu,
    EditorContent,
    FloatingMenu,
  } from '@tiptap/vue-3';

  import { reactive } from 'vue';

  import BaseModal from './BaseModal.vue';

  defineProps<{
    modelValue: boolean;
  }>();

  const editor = useEditor({
    extensions: [StarterKit],
  });
  const tippyOptions = reactive({ duration: 200 });
</script>

<style lang="scss" scoped>
  .editor {
    width: 500px;
    max-width: 90vw;
    height: fit-content;

    border: 1px solid white;
  }

  .controls {
    display: flex;
    justify-content: space-between;
  }
</style>
