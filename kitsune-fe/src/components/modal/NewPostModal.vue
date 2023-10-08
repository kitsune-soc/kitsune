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

      <div class="controls-post">
        {{ remainingCharacters }}
        <button class="controls-post-button">Post!</button>
      </div>
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

  import { Markdown } from 'tiptap-markdown';
  import { reactive } from 'vue';
  import { computed } from 'vue';

  import { useInstanceInfo } from '../../graphql/instance-info';
  import BaseModal from './BaseModal.vue';

  defineProps<{
    modelValue: boolean;
  }>();

  const editor = useEditor({
    extensions: [Markdown, StarterKit],
  });
  const tippyOptions = reactive({ duration: 200 });

  const instanceData = useInstanceInfo();
  const remainingCharacters = computed(() => {
    if (instanceData.value) {
      const markdownText = editor.value?.storage.markdown.getMarkdown();
      return instanceData.value.characterLimit - markdownText.length;
    }
  });
</script>

<style lang="scss" scoped>
  .editor {
    width: 500px;
    max-width: 90vw;
    height: fit-content;
    border: 1px solid white;

    padding: 0 1em;
    margin-bottom: 1em;
  }

  .controls {
    display: flex;
    justify-content: space-between;
  }
</style>
