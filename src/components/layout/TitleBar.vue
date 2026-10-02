<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import {
  appWindow,
  minimize,
  toggleMaximize,
  closeWindow,
  isMaximized,
  onMaximizeChange,
} from '../../api/window';

const maximized = ref(false);
let unlisten: (() => void) | null = null;

onMounted(async () => {
  maximized.value = await isMaximized();
  unlisten = await onMaximizeChange((m) => {
    maximized.value = m;
  });
});

onBeforeUnmount(() => {
  unlisten?.();
});

// 左键按住空白区域拖动窗口；点在按钮上不触发
async function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  if ((e.target as HTMLElement).closest('button')) return;
  await appWindow?.startDragging();
}

// 双击标题栏在最大化/还原之间切换
async function onDblClick(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button')) return;
  await toggleMaximize();
}
</script>

<template>
  <div class="titlebar" @mousedown="onMouseDown" @dblclick="onDblClick">
    <div class="left">
      <span class="dot" />
      <span class="name">Tool Manager</span>
    </div>
    <div class="controls">
      <button class="ctrl" title="最小化" @click="minimize">
        <svg width="11" height="11" viewBox="0 0 11 11">
          <path d="M1 5.5h9" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
      <button class="ctrl" :title="maximized ? '还原' : '最大化'" @click="toggleMaximize">
        <svg v-if="!maximized" width="11" height="11" viewBox="0 0 11 11">
          <rect x="1" y="1" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
        <svg v-else width="11" height="11" viewBox="0 0 11 11">
          <rect x="1.5" y="3" width="6.5" height="6.5" fill="none" stroke="currentColor" stroke-width="1" />
          <path d="M3.5 3V1.5h6V8H8" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
      <button class="ctrl danger" title="关闭" @click="closeWindow">
        <svg width="11" height="11" viewBox="0 0 11 11">
          <path d="M1 1l9 9M10 1l-9 9" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px 0 12px;
  color: var(--titlebar-fg);
  font-size: 12px;
  user-select: none;
  background: var(--titlebar-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--titlebar-border);
}
.left {
  display: flex;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}
.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: linear-gradient(135deg, #2f8cff, #a855f7);
}
.name {
  font-weight: 500;
  letter-spacing: 0.3px;
}
.controls {
  display: flex;
  gap: 2px;
}
.ctrl {
  width: 38px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--titlebar-fg);
  border-radius: 4px;
  transition: background 0.15s, color 0.15s;
}
.ctrl:hover {
  background: var(--titlebar-hover);
}
.ctrl.danger:hover {
  background: #e81123;
  color: #fff;
}
</style>
