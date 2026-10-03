<script setup lang="ts">
/**
 * 全局反馈弹窗宿主：在 App 根部挂载一次即可。
 * confirm 与 alert 全部经由这里渲染，样式随主题弹窗变量（透明度/模糊）。
 */
import BaseDialog from './BaseDialog.vue';
import { useFeedback } from '../../composables/useFeedback';

const { current, settle } = useFeedback();
</script>

<template>
  <BaseDialog
    v-if="current"
    :key="current.title + current.message"
    :title="current.title"
    role="alertdialog"
    top="22vh"
    width="min(420px, 92vw)"
    @close="settle(current.kind === 'alert')"
  >
    <p class="feedback-message">{{ current.message }}</p>

    <template #footer>
      <template v-if="current.kind === 'confirm'">
        <button class="tm-btn" @click="settle(false)">{{ current.cancelText }}</button>
        <button
          class="tm-btn"
          :class="current.danger ? 'danger' : 'primary'"
          @click="settle(true)"
        >
          {{ current.confirmText }}
        </button>
      </template>
      <button v-else class="tm-btn primary" @click="settle(true)">
        {{ current.confirmText }}
      </button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.feedback-message {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
