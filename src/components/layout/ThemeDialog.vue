<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount } from 'vue';
import { useSettingsStore, BUILTIN_BGS, bgUrl } from '../../stores/settings';

const emit = defineEmits<{ close: [] }>();
const settings = useSettingsStore();

const modes = [
  { value: 'default', label: '默认', tip: '保持原样' },
  { value: 'light', label: '浅色主题', tip: '浅底深字' },
  { value: 'dark', label: '深色主题', tip: '深底浅字' },
] as const;

const builtinList = computed(() =>
  settings.mode === 'default' ? [] : BUILTIN_BGS[settings.mode].map((n) => `builtin:${n}`),
);
const customList = computed(() =>
  settings.customBgs.filter((c) => c.theme === settings.mode).map((c) => c.path),
);

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  document.body.style.overflow = 'hidden';
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  document.body.style.overflow = '';
});
</script>

<template>
  <Teleport to="body">
    <div class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog-header">
          <span>主题设置</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <label class="field-label">主题大类</label>
          <div class="mode-row">
            <button
              v-for="m in modes"
              :key="m.value"
              class="mode-card"
              :class="{ active: settings.mode === m.value }"
              @click="settings.setMode(m.value)"
            >
              <span class="mode-name">{{ m.label }}</span>
              <span class="mode-tip">{{ m.tip }}</span>
            </button>
          </div>

          <template v-if="settings.themed">
            <label class="field-label">
              页面透明度 <span class="opacity-val">{{ Math.round(settings.opacity * 100) }}%</span>
            </label>
            <input
              class="opacity-slider"
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="settings.opacity"
              @input="settings.setOpacity(Number(($event.target as HTMLInputElement).value))"
            />
            <p class="hint">按钮自动 = 页面+25%，卡片与目录 = 页面+15%（不超过 100%）</p>

            <label class="field-label">背景图片（{{ settings.mode === 'light' ? '浅色' : '深色' }}类）</label>
            <div class="bg-grid">
              <div
                v-for="b in builtinList"
                :key="b"
                class="bg-cell"
                :class="{ active: settings.bg === b }"
                @click="settings.selectBg(b)"
              >
                <img :src="bgUrl(b)" :alt="b" />
              </div>
              <div
                v-for="p in customList"
                :key="p"
                class="bg-cell"
                :class="{ active: settings.bg === p }"
                :title="p"
                @click="settings.selectBg(p)"
              >
                <img :src="bgUrl(p)" :alt="p" />
                <button
                  class="bg-remove"
                  title="从列表移除"
                  @click.stop="settings.removeCustomBg(p)"
                >
                  ✕
                </button>
              </div>
              <button class="bg-cell import" title="从本地导入图片" @click="settings.importBg()">
                ＋<span class="import-text">导入</span>
              </button>
            </div>
          </template>
          <p v-else class="hint">当前为默认外观。选择浅色或深色主题后，可调整页面透明度并设置背景图片。</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
  z-index: 1000;
}
.dialog {
  background: #fff;
  --text: #1f2329;
  --text-sub: #6b7280;
  width: min(520px, 94vw);
  border-radius: 8px;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.2);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  border-bottom: 1px solid #e5e7eb;
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 16px;
  color: var(--text-sub);
  padding: 4px 8px;
  border-radius: 4px;
}
.close-btn:hover {
  background: #f0f1f3;
  color: var(--text);
}
.dialog-body {
  padding: 16px 18px 20px;
}
.field-label {
  display: block;
  font-size: 13px;
  color: var(--text);
  margin-bottom: 8px;
  margin-top: 16px;
}
.field-label:first-of-type {
  margin-top: 0;
}
.mode-row {
  display: flex;
  gap: 8px;
}
.mode-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 10px 6px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #fafbfc;
}
.mode-card:hover {
  border-color: #c6cdd6;
}
.mode-card.active {
  border-color: #2f8cff;
  background: #eef4ff;
}
.mode-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.mode-tip {
  font-size: 11px;
  color: var(--text-sub);
}
.opacity-val {
  color: var(--text-sub);
  font-weight: 400;
}
.opacity-slider {
  width: 100%;
  accent-color: #2f8cff;
}
.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-sub);
  line-height: 1.5;
}
.bg-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.bg-cell {
  position: relative;
  aspect-ratio: 16 / 10;
  border-radius: 6px;
  overflow: hidden;
  border: 2px solid transparent;
  cursor: pointer;
  background: #f0f1f3;
}
.bg-cell img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.bg-cell:hover {
  border-color: #c6cdd6;
}
.bg-cell.active {
  border-color: #2f8cff;
}
.bg-cell.import {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 20px;
  color: var(--text-sub);
  border: 1px dashed #c6cdd6;
  background: transparent;
}
.bg-cell.import:hover {
  color: #2f8cff;
  border-color: #2f8cff;
}
.import-text {
  font-size: 11px;
}
.bg-remove {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  font-size: 10px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.bg-remove:hover {
  background: #d33;
}
</style>
