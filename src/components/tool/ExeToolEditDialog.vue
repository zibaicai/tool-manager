<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Tool } from '../../types';
import { useCategoryStore } from '../../stores/categories';
import { useCmdToolsStore } from '../../stores/cmdTools';
import { useExeToolsStore } from '../../stores/exeTools';
import { TOOL_TYPES } from '../../constants';
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
const weight = ref(props.tool.weight ?? 0);
const admin = ref(props.tool.admin ?? false);
const stopPath = ref(props.tool.stopPath ?? '');
const stopAdmin = ref(props.tool.stopAdmin ?? false);

/** 全部业务分组（侧栏所见即下拉所得，不再区分 scan/manual） */
const groups = computed(() => categoryStore.sortedCategoryGroups);
/** 工具当前所在组的 key；不在任何已配置分类下（自动归属）时为空串 */
const initialGroupKey = computed(
  () => groups.value.find((g) => g.ids.includes(props.tool.categoryId))?.key ?? '',
);
const selectedGroupKey = ref(initialGroupKey.value);
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
        weight.value || 0,
      );
    } else {
      await cmdStore.updateCmd(props.tool.id, title.value, desc.value, weight.value || 0);
      // 按"业务分组"比较归属是否变化：组内 scan/manual 成员切换不算用户改归属
      if (selectedGroupKey.value !== initialGroupKey.value) {
        const target =
          groups.value.find((g) => g.key === selectedGroupKey.value) ?? null;
        await cmdStore.assignCmd(props.tool.id, target?.primaryId ?? null);
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
      v-model:weight="weight"
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
      <select v-model="selectedGroupKey" class="tm-text-input">
        <option value="">自动（按目录扫描规则）</option>
        <option v-for="g in groups" :key="g.key" :value="g.key">{{ g.name }}</option>
      </select>
      <p class="tm-hint">可分配到任意业务目录；扫描型目录要求工具位于其扫描根下</p>

      <label class="tm-field-label">排序权重</label>
      <input
        v-model.number="weight"
        class="tm-text-input"
        type="number"
        step="1"
        placeholder="0"
        @keyup.enter="submit"
      />
      <p class="tm-hint">数值越大在目录内越靠前；同权重保持原有顺序，默认 0</p>
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
