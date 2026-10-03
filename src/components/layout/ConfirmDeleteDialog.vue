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
const toolCount = computed(() => store.toolsOf(props.categoryId).length);
const { submitting, error, run } = useAsyncSubmit();

async function confirmDelete() {
  if (await run(() => store.deleteCat(props.categoryId))) emit('close');
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
      确定删除目录 <strong class="cat-name">「{{ cat?.name }}」</strong> 吗？
    </p>
    <p class="meta">
      类型：{{ cat?.type === CATEGORY_TYPES.MANUAL ? 'EXE 手动录入' : 'CMD 自动扫描' }}
      <span v-if="toolCount > 0"> · 当前含 {{ toolCount }} 个工具</span>
    </p>
    <p v-if="cat?.type === CATEGORY_TYPES.MANUAL && toolCount > 0" class="warn">
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
