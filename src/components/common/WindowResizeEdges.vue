<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { isMaximized, onMaximizeChange, startResize } from '../../api/window';
import type { ResizeDirection } from '../../api/window';

/**
 * 无边框窗口的 8 向边缘缩放热区（4 边 + 4 角）。
 * - Teleport 到 body，容器 pointer-events:none，仅热区接收鼠标，不挡正常点击；
 * - 最大化时禁用缩放；
 * - 任意弹窗（.tm-overlay）打开时，JS 热区实时查 DOM 拒绝触发，保证弹窗时绝不误缩放。
 */
const maximized = ref(false);
let unlisten: (() => void) | null = null;

/** 是否有弹窗打开（.tm-overlay：居中弹窗/文档抽屉/确认框等所有 BaseDialog） */
function hasOverlay(): boolean {
  return document.querySelector('.tm-overlay') !== null;
}

onMounted(() => {
  // 独立 catch：任何一步失败都不影响 mousedown 时的实时判断
  isMaximized()
    .then((m) => {
      maximized.value = m;
    })
    .catch(() => {});
  onMaximizeChange((m) => {
    maximized.value = m;
  })
    .then((un) => {
      unlisten = un;
    })
    .catch(() => {});
});

onBeforeUnmount(() => {
  unlisten?.();
});

async function beginResize(e: MouseEvent, direction: ResizeDirection) {
  if (e.button !== 0 || hasOverlay()) return;
  // 实时再确认一次最大化状态，不依赖监听是否成功注册
  if (await isMaximized().catch(() => false)) return;
  e.preventDefault();
  e.stopPropagation();
  await startResize(direction);
}
</script>

<template>
  <Teleport to="body">
    <div
      class="tm-resize-layer"
      :class="{ disabled: maximized }"
      aria-hidden="true"
    >
      <!-- 四边 -->
      <div class="edge edge-n" @mousedown="beginResize($event, 'North')" />
      <div class="edge edge-s" @mousedown="beginResize($event, 'South')" />
      <div class="edge edge-w" @mousedown="beginResize($event, 'West')" />
      <div class="edge edge-e" @mousedown="beginResize($event, 'East')" />
      <!-- 四角（后渲染，优先于边） -->
      <div class="corner corner-nw" @mousedown="beginResize($event, 'NorthWest')" />
      <div class="corner corner-ne" @mousedown="beginResize($event, 'NorthEast')" />
      <div class="corner corner-sw" @mousedown="beginResize($event, 'SouthWest')" />
      <div class="corner corner-se" @mousedown="beginResize($event, 'SouthEast')" />
    </div>
  </Teleport>
</template>

<style scoped>
.tm-resize-layer {
  position: fixed;
  inset: 0;
  z-index: 900;
  pointer-events: none;
}
.tm-resize-layer.disabled {
  display: none;
}
.edge {
  position: absolute;
  pointer-events: auto;
}
.edge-n,
.edge-s {
  left: 8px;
  right: 8px;
  height: 6px;
  cursor: ns-resize;
}
.edge-n {
  top: 0;
}
.edge-s {
  bottom: 0;
}
.edge-w,
.edge-e {
  top: 8px;
  bottom: 8px;
  width: 6px;
  cursor: ew-resize;
}
.edge-w {
  left: 0;
}
.edge-e {
  right: 0;
}
.corner {
  position: absolute;
  width: 10px;
  height: 10px;
  pointer-events: auto;
}
.corner-nw {
  top: 0;
  left: 0;
  cursor: nwse-resize;
}
.corner-ne {
  top: 0;
  right: 0;
  cursor: nesw-resize;
}
.corner-sw {
  bottom: 0;
  left: 0;
  cursor: nesw-resize;
}
.corner-se {
  bottom: 0;
  right: 0;
  cursor: nwse-resize;
}
</style>
