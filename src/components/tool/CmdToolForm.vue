<script setup lang="ts">
import { ref, computed } from 'vue';
import { useCategoryStore } from '../../stores/categories';
import { useCmdToolsStore } from '../../stores/cmdTools';
import { registerCmdTool, pickDir } from '../../api/fs';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import BaseDialog from '../common/BaseDialog.vue';

const emit = defineEmits<{ close: [] }>();
const categoryStore = useCategoryStore();
const cmdStore = useCmdToolsStore();

const categoryName = computed(() => categoryStore.activeGroup?.name ?? '当前分类');
const targetCategoryId = computed(() => categoryStore.exeTargetCategoryId);

const dirPath = ref('');
const weight = ref(0);
const { submitting, error, run } = useAsyncSubmit();

async function browse() {
  try {
    const picked = await pickDir();
    if (picked) dirPath.value = picked;
  } catch (e) {
    error.value = '打开目录选择框失败: ' + e;
  }
}

async function submit() {
  if (!dirPath.value.trim()) {
    error.value = '请填写或选择工具目录路径';
    return;
  }
  const ok = await run(async () => {
    const tool = await registerCmdTool(
      dirPath.value.trim(),
      targetCategoryId.value,
      weight.value || 0,
    );
    cmdStore.tools.push(tool);
  });
  if (ok) emit('close');
}
</script>

<template>
  <BaseDialog
    :title="`添加 CMD 工具到「${categoryName}」`"
    width="min(560px, 92vw)"
    @close="emit('close')"
  >
    <label class="tm-field-label">工具目录路径</label>
    <div class="tm-path-row">
      <input
        v-model="dirPath"
        class="tm-text-input"
        type="text"
        placeholder="D:\Program\Heavenly Fox\tools\gui_scan\fscan"
        spellcheck="false"
        @keyup.enter="submit"
      />
      <button class="tm-browse-btn" @click="browse">浏览...</button>
    </div>
    <p class="tm-hint">
      适用于不在扫描根下的命令行工具目录（如天狐工具箱等外部工具源）。目录内需含
      .exe/.bat/.ps1/.py 等可执行文件
    </p>

    <label class="tm-field-label">排序权重（可选）</label>
    <input
      v-model.number="weight"
      class="tm-text-input"
      type="number"
      placeholder="0"
    />
    <p class="tm-hint">数值越大在分类内越靠前，默认 0</p>

    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="submit">
        {{ submitting ? '添加中...' : '添加' }}
      </button>
    </template>
  </BaseDialog>
</template>
