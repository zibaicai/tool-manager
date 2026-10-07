<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useCategoryStore } from '../../stores/categories';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import { alert } from '../../composables/useFeedback';
import BaseDialog from '../common/BaseDialog.vue';

const emit = defineEmits<{ close: [] }>();
const store = useCategoryStore();

const scanRoot = ref(store.scanRoot);
const { submitting, error, run } = useAsyncSubmit();

/** 调出系统目录选择框 */
async function browse() {
  const selected = await open({
    title: '选择 CMD 工具扫描根目录',
    directory: true,
    multiple: false,
    defaultPath: scanRoot.value || undefined,
  });
  if (typeof selected === 'string') scanRoot.value = selected;
}

async function save() {
  if (await run(() => store.saveScanRoot(scanRoot.value))) emit('close');
}

/** 在系统默认浏览器中打开在线使用说明书 */
async function openManual() {
  try {
    await invoke('open_manual');
  } catch (e) {
    await alert('打开失败: ' + e);
  }
}
</script>

<template>
  <BaseDialog title="设置" width="min(520px, 94vw)" @close="emit('close')">
    <label class="tm-field-label">CMD 工具扫描根目录（scanRoot）</label>
    <div class="tm-path-row">
      <input
        v-model="scanRoot"
        class="tm-text-input"
        type="text"
        placeholder="例如 D:\ScriptingTool\cmd_tool"
        spellcheck="false"
        @keyup.enter="save"
      />
      <button class="tm-browse-btn" type="button" title="浏览选择目录" @click="browse">
        浏览…
      </button>
    </div>
    <p class="tm-hint">
      CMD 类工具统一存放的汇总目录；未单独配置扫描目录的分类都会在此目录下扫描。
      保存后自动重新扫描，清空后保存可取消该设置。
    </p>

    <div class="manual-section">
      <label class="tm-field-label">使用说明书</label>
      <button class="manual-link" type="button" @click="openManual">查看使用说明 ↗</button>
      <p class="tm-hint">在浏览器中打开完整说明书（在线文档，需要网络）</p>
    </div>

    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="save">
        {{ submitting ? '保存中...' : '保存' }}
      </button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.manual-section {
  margin-top: 18px;
  padding-top: 14px;
  border-top: 1px solid var(--border, #333);
}
.manual-link {
  display: inline-block;
  padding: 2px 0;
  background: none;
  border: none;
  color: var(--primary, #4f8cff);
  font-size: 13px;
  text-decoration: underline;
  cursor: pointer;
}
.manual-link:hover {
  filter: brightness(1.2);
}
</style>
