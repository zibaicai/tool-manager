<script setup lang="ts">
import { computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '../../stores/settings';
import { BUILTIN_BGS, bgUrl } from '../../utils/themeBg';
import BaseDialog from '../common/BaseDialog.vue';
import {
  BUILTIN_BG_PREFIX,
  DIALOG_BLUR_SLIDER_MAX,
  OPACITY_MAX,
  OPACITY_MIN,
  THEME_MODES,
} from '../../constants';

const emit = defineEmits<{ close: [] }>();
const settings = useSettingsStore();

const modes = [
  { value: THEME_MODES.DEFAULT, label: '默认', tip: '保持原样' },
  { value: THEME_MODES.LIGHT, label: '浅色主题', tip: '浅底深字' },
  { value: THEME_MODES.DARK, label: '深色主题', tip: '深底浅字' },
] as const;

const builtinList = computed(() =>
  settings.mode === THEME_MODES.DEFAULT
    ? []
    : BUILTIN_BGS[settings.mode].map((n) => `${BUILTIN_BG_PREFIX}${n}`),
);
const customList = computed(() =>
  settings.customBgs.filter((c) => c.theme === settings.mode).map((c) => c.path),
);

/** 文件选择器留在组件层（store 只管状态与持久化） */
async function importBg() {
  if (settings.mode === THEME_MODES.DEFAULT) return;
  const picked = await open({
    title: '选择背景图片',
    multiple: false,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'svg'] }],
  });
  if (picked && !Array.isArray(picked)) await settings.addCustomBg(picked);
}
</script>

<template>
  <BaseDialog title="主题设置" top="12vh" width="min(520px, 94vw)" @close="emit('close')">
    <label class="tm-field-label">主题大类</label>
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

    <label class="tm-field-label">
      弹窗不透明度 <span class="opacity-val">{{ Math.round(settings.dialogOpacity * 100) }}%</span>
    </label>
    <input
      class="opacity-slider"
      type="range"
      :min="0.3"
      :max="OPACITY_MAX"
      step="0.05"
      :value="settings.dialogOpacity"
      @input="settings.setDialogOpacity(Number(($event.target as HTMLInputElement).value))"
    />

    <label class="tm-field-label">
      弹窗背景模糊 <span class="opacity-val">{{ Math.round(settings.dialogBlur) }}px</span>
    </label>
    <input
      class="opacity-slider"
      type="range"
      :min="0"
      :max="DIALOG_BLUR_SLIDER_MAX"
      step="1"
      :value="settings.dialogBlur"
      @input="settings.setDialogBlur(Number(($event.target as HTMLInputElement).value))"
    />
    <p class="tm-hint">对所有弹窗生效（添加/编辑工具、删除确认、文档等），拖动时本窗口实时预览。</p>

    <template v-if="settings.themed">
      <label class="tm-field-label">
        页面透明度 <span class="opacity-val">{{ Math.round(settings.opacity * 100) }}%</span>
      </label>
      <input
        class="opacity-slider"
        type="range"
        :min="OPACITY_MIN"
        :max="OPACITY_MAX"
        step="0.05"
        :value="settings.opacity"
        @input="settings.setOpacity(Number(($event.target as HTMLInputElement).value))"
      />
      <p class="tm-hint">按钮自动 = 页面+25%，卡片与目录 = 页面+15%（不超过 100%）</p>

      <label class="tm-field-label">
        背景图片（{{ settings.mode === THEME_MODES.LIGHT ? '浅色' : '深色' }}类）
      </label>
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
          <button class="bg-remove" title="从列表移除" @click.stop="settings.removeCustomBg(p)">
            ✕
          </button>
        </div>
        <button class="bg-cell import" title="从本地导入图片" @click="importBg">
          ＋<span class="import-text">导入</span>
        </button>
      </div>
    </template>
    <p v-else class="tm-hint">当前为默认外观。选择浅色或深色主题后，可调整页面透明度并设置背景图片。</p>
  </BaseDialog>
</template>

<style scoped>
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
  border: 1px solid var(--dialog-border);
  border-radius: 8px;
  background: var(--dialog-soft);
}
.mode-card:hover {
  border-color: var(--input-border);
}
.mode-card.active {
  border-color: var(--primary);
  background: rgba(47, 140, 255, 0.16);
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
  background: var(--dialog-hover);
}
.bg-cell img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.bg-cell:hover {
  border-color: var(--input-border);
}
.bg-cell.active {
  border-color: var(--primary);
}
.bg-cell.import {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 20px;
  color: var(--text-sub);
  border: 1px dashed var(--input-border);
  background: transparent;
}
.bg-cell.import:hover {
  color: var(--primary);
  border-color: var(--primary);
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
