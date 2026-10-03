<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Tool } from '../../types';
import { useCategoryStore } from '../../stores/categories';
import { useCmdToolsStore } from '../../stores/cmdTools';
import { useExeToolsStore } from '../../stores/exeTools';
import { CATEGORY_TYPES, TOOL_TYPES } from '../../constants';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import BaseDialog from '../common/BaseDialog.vue';
import ExeToolFields from './ExeToolFields.vue';

const props = defineProps<{ tool: Tool }>();
const emit = defineEmits<{ close: [] }>();
const categoryStore = useCategoryStore();
const cmdStore = useCmdToolsStore();
const exeStore = useExeToolsStore();

const title = ref(props.tool.title);
const desc = ref(props.tool.desc ?? '');
const admin = ref(props.tool.admin ?? false);
const stopPath = ref(props.tool.stopPath ?? '');
const stopAdmin = ref(props.tool.stopAdmin ?? false);
const assignedCat = ref(props.tool.categoryId);
const scanCats = computed(() =>
  categoryStore.sortedCategories.filter((c) => c.type === CATEGORY_TYPES.SCAN),
);
const { submitting, error, run } = useAsyncSubmit();

async function submit() {
  const ok = await run(async () => {
    if (props.tool.type === TOOL_TYPES.EXE) {
      await exeStore.updateExe(
        props.tool.id,
        title.value,
        desc.value,
        admin.value,
        stopPath.value,
        stopAdmin.value,
      );
    } else {
      await cmdStore.updateCmd(props.tool.id, title.value, desc.value);
      if (assignedCat.value !== props.tool.categoryId) {
        await cmdStore.assignCmd(props.tool.id, assignedCat.value || null);
        // 归属变更影响扫描分区，需重新扫描
        await categoryStore.refresh();
      }
    }
  });
  if (ok) emit('close');
}
</script>

<template>
  <BaseDialog :title="`编辑「${tool.title}」`" width="min(480px, 92vw)" @close="emit('close')">
    <ExeToolFields
      v-if="tool.type === TOOL_TYPES.EXE"
      v-model:title="title"
      v-model:desc="desc"
      v-model:admin="admin"
      v-model:stop-path="stopPath"
      v-model:stop-admin="stopAdmin"
    />

    <template v-else>
      <label class="tm-field-label">工具标题</label>
      <input v-model="title" class="tm-text-input" type="text" placeholder="留空恢复自动取名" />

      <label class="tm-field-label">副标题 · 用途描述</label>
      <input
        v-model="desc"
        class="tm-text-input"
        type="text"
        placeholder="一句话说明工具用途，展示在标题下方"
        @keyup.enter="submit"
      />

      <label class="tm-field-label">所属目录</label>
      <select v-model="assignedCat" class="tm-text-input">
        <option value="">自动（按目录扫描规则）</option>
        <option v-for="c in scanCats" :key="c.id" :value="c.id">{{ c.name }}</option>
      </select>
      <p class="tm-hint">仅可选择 CMD 自动扫描类目录，且工具需位于目标目录的扫描根路径下</p>
    </template>

    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="submit">
        {{ submitting ? '保存中...' : '保存' }}
      </button>
    </template>
  </BaseDialog>
</template>
