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

const emit = defineEmits<{ settings: []; search: [] }>();

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
      <button class="ctrl search" title="搜索工具 (Ctrl+K)" @click="emit('search')">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="11" cy="11" r="7" />
          <path d="M21 21l-4.35-4.35" />
        </svg>
      </button>
      <button class="ctrl settings" title="设置" @click="emit('settings')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
      </button>
      <span class="sep" />
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
  align-items: center;
  gap: 2px;
}
.sep {
  width: 1px;
  height: 14px;
  background: var(--titlebar-border);
  margin: 0 6px 0 4px;
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
.ctrl.settings,
.ctrl.search {
  width: 30px;
}
.ctrl.settings:hover,
.ctrl.search:hover {
  color: var(--primary, #2f8cff);
}
.ctrl.danger:hover {
  background: #e81123;
  color: #fff;
}
</style>
