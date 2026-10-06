<script setup lang="ts">
import { computed } from 'vue';
import { useCategoryStore } from '../../stores/categories';
import { CATEGORY_TYPES } from '../../constants';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import BaseDialog from '../common/BaseDialog.vue';

const props = defineProps<{ categoryId: string }>();
const emit = defineEmits<{ close: [] }>();
const store = useCategoryStore();

const cat = computed(() => store.categories.find((c) => c.id === props.categoryId));
/** 当前分类所在展示组（含同名 scan + manual 合并） */
const group = computed(() =>
  store.sortedCategoryGroups.find((g) => g.ids.includes(props.categoryId)),
);
const memberIds = computed(() => group.value?.ids ?? [props.categoryId]);
const memberCats = computed(() =>
  memberIds.value
    .map((id) => store.categories.find((c) => c.id === id))
    .filter(Boolean),
);
const toolCount = computed(() =>
  memberIds.value.reduce((n, id) => n + store.toolsOf(id).length, 0),
);
const hasManualWithTools = computed(() =>
  memberCats.value.some(
    (c: any) => c?.type === CATEGORY_TYPES.MANUAL && store.toolsOf(c.id).length > 0,
  ),
);
const { submitting, error, run } = useAsyncSubmit();

async function confirmDelete() {
  const ids = group.value?.ids ?? [props.categoryId];
  if (
    await run(async () => {
      for (const id of ids) await store.deleteCat(id);
    })
  )
    emit('close');
}
</script>

<template>
  <BaseDialog
    title="删除目录"
    role="alertdialog"
    top="22vh"
    width="min(420px, 92vw)"
    @close="emit('close')"
  >
    <p class="lead">
      确定删除目录 <strong class="cat-name">「{{ group?.name ?? cat?.name }}」</strong> 吗？
    </p>
    <p class="meta">
      <template v-if="(group?.ids?.length ?? 0) > 1">
        该目录由 {{ group!.ids.length }} 个分类合并显示，将一并删除：
        {{ memberCats.map((c: any) => c?.name).join(' + ') }}
      </template>
      <template v-else>
        类型：{{ cat?.type === CATEGORY_TYPES.MANUAL ? 'EXE 手动录入' : 'CMD 自动扫描' }}
      </template>
      <span v-if="toolCount > 0"> · 当前含 {{ toolCount }} 个工具</span>
    </p>
    <p v-if="hasManualWithTools" class="warn">
      删除后其下 EXE 录入将不再显示（数据保留在 exe-tools.json，重新录入可恢复）。
    </p>
    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn danger" :disabled="submitting" @click="confirmDelete">
        {{ submitting ? '删除中...' : '删除' }}
      </button>
    </template>
  </BaseDialog>
</template>

<style scoped>
.lead {
  font-size: 14px;
  color: var(--text);
  margin: 0;
}
.cat-name {
  color: #c0392b;
}
.meta {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-sub);
}
.warn {
  margin-top: 10px;
  padding: 8px 10px;
  font-size: 12px;
  line-height: 1.5;
  color: #9a6a00;
  background: #fdf6e3;
  border-radius: 6px;
}
</style>
