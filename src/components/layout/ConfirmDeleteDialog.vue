<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useToolsStore } from '../../stores/tools';

const props = defineProps<{ categoryId: string }>();
const emit = defineEmits<{ close: [] }>();
const store = useToolsStore();

const cat = computed(() => store.categories.find((c) => c.id === props.categoryId));
const toolCount = computed(
  () => store.tools.filter((t) => t.categoryId === props.categoryId).length,
);
const error = ref('');
const submitting = ref(false);

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

async function confirmDelete() {
  error.value = '';
  submitting.value = true;
  try {
    await store.deleteCat(props.categoryId);
    emit('close');
  } catch (e) {
    error.value = String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="overlay" @click.self="emit('close')">
      <div class="dialog" role="alertdialog" aria-modal="true">
        <header class="dialog-header">
          <span>删除目录</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <p class="lead">
            确定删除目录 <strong class="cat-name">「{{ cat?.name }}」</strong> 吗？
          </p>
          <p class="meta">
            类型：{{ cat?.type === 'manual' ? 'EXE 手动录入' : 'CMD 自动扫描' }}
            <span v-if="toolCount > 0"> · 当前含 {{ toolCount }} 个工具</span>
          </p>
          <p v-if="cat?.type === 'manual' && toolCount > 0" class="warn">
            删除后其下 EXE 录入将不再显示（数据保留在 exe-tools.json，重新录入可恢复）。
          </p>
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <footer class="dialog-footer">
          <button class="btn" @click="emit('close')">取消</button>
          <button class="btn danger" :disabled="submitting" @click="confirmDelete">
            {{ submitting ? '删除中...' : '删除' }}
          </button>
        </footer>
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
  padding-top: 22vh;
  z-index: 1000;
}
.dialog {
  background: #fff;
  --text: #1f2329;
  --text-sub: #6b7280;
  width: min(420px, 92vw);
  border-radius: 8px;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.2);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  border-bottom: 1px solid var(--card-border);
  font-size: 15px;
  font-weight: 600;
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
  padding: 16px 18px;
}
.lead {
  font-size: 14px;
  color: var(--text);
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
.error {
  margin-top: 8px;
  font-size: 12px;
  color: #d33;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 18px;
  border-top: 1px solid var(--card-border);
}
.btn {
  padding: 7px 18px;
  font-size: 13px;
  border: 1px solid var(--card-border);
  background: #fff;
  border-radius: 6px;
}
.btn:hover:not(:disabled) {
  background: #f3f5f8;
}
.btn.danger {
  background: #d33;
  color: #fff;
  border-color: #d33;
}
.btn.danger:hover:not(:disabled) {
  background: #b92c2c;
}
.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
