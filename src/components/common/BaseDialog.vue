<script setup lang="ts">
import { useEscapeLock } from '../../composables/useEscapeLock';

/**
 * 全部弹窗/抽屉的统一外壳：Teleport + 遮罩 + 头部标题/关闭 + 可选底部按钮区。
 * 键盘 Esc 与 body 滚动锁由 useEscapeLock 统一处理；
 * 所有外观 CSS 集中在本文件的全局样式块（锚定 .tm-overlay，不污染页面其他元素）。
 *
 * variant：
 * - center：居中浮层（设置/表单/确认类弹窗），top 控制距顶距离
 * - panel：左侧全高抽屉（文档面板），width 控制抽屉宽度
 */
const props = withDefaults(
  defineProps<{
    title: string;
    variant?: 'center' | 'panel';
    /** center 变体距视口顶部的距离 */
    top?: string;
    /** 浮层/抽屉宽度（CSS 值） */
    width?: string;
    /** ARIA role：普通对话框 dialog，强确认用 alertdialog */
    role?: 'dialog' | 'alertdialog';
    /** 点击遮罩空白处是否关闭 */
    closeOnOverlay?: boolean;
  }>(),
  {
    variant: 'center',
    top: '18vh',
    width: 'min(520px, 94vw)',
    role: 'dialog',
    closeOnOverlay: true,
  },
);

const emit = defineEmits<{ close: [] }>();
useEscapeLock(() => emit('close'));

function onOverlayClick() {
  if (props.closeOnOverlay) emit('close');
}
</script>

<template>
  <Teleport to="body">
    <div
      class="tm-overlay"
      :class="variant"
      :style="variant === 'center' ? { paddingTop: top } : undefined"
      @click.self="onOverlayClick"
    >
      <div class="tm-dialog" :role="role" aria-modal="true" :style="{ width }">
        <header class="tm-dialog-header">
          <span class="tm-dialog-title">{{ title }}</span>
          <button class="tm-close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="tm-dialog-body">
          <slot />
        </div>

        <footer v-if="$slots.footer" class="tm-dialog-footer">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style>
/* ===== 弹窗外壳（锚定 .tm-overlay，避免与卡片等其他 .btn 冲突） ===== */
.tm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  z-index: 1000;
}
.tm-overlay.center {
  justify-content: center;
  align-items: flex-start;
}
.tm-overlay.panel {
  justify-content: flex-start;
}

.tm-dialog {
  background: var(--dialog-bg);
  backdrop-filter: blur(var(--dialog-blur));
  -webkit-backdrop-filter: blur(var(--dialog-blur));
  --text: var(--dialog-fg);
  --text-sub: var(--dialog-fg-sub);
  color: var(--text);
  border-radius: 8px;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  max-height: 78vh;
}
.tm-overlay.panel .tm-dialog {
  height: 100vh;
  max-height: 100vh;
  border-radius: 0;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.18);
}

.tm-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  border-bottom: 1px solid var(--dialog-border);
  font-size: 15px;
  font-weight: 600;
  flex-shrink: 0;
}
.tm-overlay.panel .tm-dialog-header {
  padding: 12px 20px;
}
.tm-close-btn {
  border: none;
  background: transparent;
  font-size: 16px;
  color: var(--text-sub);
  padding: 4px 8px;
  border-radius: 4px;
  flex-shrink: 0;
}
.tm-close-btn:hover {
  background: var(--dialog-hover);
  color: var(--text);
}

.tm-dialog-body {
  padding: 16px 18px;
  overflow-y: auto;
  min-height: 0;
}
.tm-overlay.panel .tm-dialog-body {
  padding: 0;
}
.tm-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 18px;
  border-top: 1px solid var(--dialog-border);
  flex-shrink: 0;
}

/* ===== 弹窗内表单基元 ===== */
.tm-field-label {
  display: block;
  font-size: 13px;
  color: var(--text);
  margin: 12px 0 6px;
}
.tm-field-label:first-of-type,
.tm-field-label:first-child {
  margin-top: 0;
}
.tm-text-input {
  width: 100%;
  flex: 1 1 auto;
  min-width: 0;
  box-sizing: border-box;
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  outline: none;
  background: var(--input-bg);
  color: var(--text);
}
.tm-text-input:focus {
  border-color: var(--primary);
}
.tm-path-row {
  display: flex;
  gap: 8px;
}
.tm-browse-btn {
  flex-shrink: 0;
  padding: 7px 14px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
  color: var(--text);
}
.tm-browse-btn:hover {
  background: var(--dialog-btn-hover);
}
.tm-hint {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--text-sub);
  line-height: 1.5;
}
.tm-check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}
.tm-error {
  margin: 8px 0 0;
  font-size: 12px;
  color: #d33;
}

/* ===== 弹窗按钮 ===== */
.tm-btn {
  padding: 7px 18px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
  color: var(--text);
}
.tm-btn:hover:not(:disabled) {
  background: var(--dialog-btn-hover);
}
.tm-btn.primary {
  background: var(--primary);
  color: #fff;
  border-color: var(--primary);
}
.tm-btn.primary:hover:not(:disabled) {
  background: var(--btn-primary-hover-bg);
}
.tm-btn.danger {
  background: #d33;
  color: #fff;
  border-color: #d33;
}
.tm-btn.danger:hover:not(:disabled) {
  background: #b92c2c;
}
.tm-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
