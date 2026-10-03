<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useCategoryStore } from '../../stores/categories';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
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
    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="save">
        {{ submitting ? '保存中...' : '保存' }}
      </button>
    </template>
  </BaseDialog>
</template>
