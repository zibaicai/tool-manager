<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useToolsStore } from '../../stores/tools';

const props = defineProps<{ mode: 'add' | 'rename' }>();
const emit = defineEmits<{ close: [] }>();
const store = useToolsStore();

const activeCat = store.categories.find((c) => c.id === store.activeCategoryId);
const name = ref(props.mode === 'rename' ? (activeCat?.name ?? '') : '');
const catType = ref<'scan' | 'manual'>('manual');
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

async function submit() {
  error.value = '';
  submitting.value = true;
  try {
    if (props.mode === 'add') {
      await store.addCat(name.value, catType.value);
    } else {
      await store.renameCat(store.activeCategoryId, name.value);
    }
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
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog-header">
          <span>{{ mode === 'add' ? '添加目录' : `重命名「${activeCat?.name ?? ''}」` }}</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <label class="field-label">目录名称</label>
          <input
            v-model="name"
            class="text-input"
            type="text"
            placeholder="左侧菜单显示的名称"
            @keyup.enter="submit"
          />

          <template v-if="mode === 'add'">
            <label class="field-label">目录类型</label>
            <select v-model="catType" class="text-input">
              <option value="manual">EXE 手动录入</option>
              <option value="scan">CMD 自动扫描（扫描 scanRoot 汇总目录）</option>
            </select>
            <p class="hint">新建的 CMD 目录初始为空，请在工具卡片的编辑菜单中通过「所属目录」分配工具</p>
          </template>
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <footer class="dialog-footer">
          <button class="btn" @click="emit('close')">取消</button>
          <button class="btn primary" :disabled="submitting" @click="submit">
            {{ submitting ? '保存中...' : '保存' }}
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
  padding-top: 18vh;
  z-index: 1000;
}
.dialog {
  background: #fff;
  --text: #1f2329;
  --text-sub: #6b7280;
  width: min(440px, 92vw);
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
.field-label {
  display: block;
  font-size: 13px;
  color: var(--text);
  margin-bottom: 6px;
  margin-top: 12px;
}
.field-label:first-of-type {
  margin-top: 0;
}
.text-input {
  width: 100%;
  box-sizing: border-box;
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--card-border);
  border-radius: 6px;
  outline: none;
  background: #fff;
}
.text-input:focus {
  border-color: var(--primary);
}
.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-sub);
  line-height: 1.5;
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
.btn.primary {
  background: var(--primary);
  color: #fff;
  border-color: var(--primary);
}
.btn.primary:hover:not(:disabled) {
  background: var(--btn-primary-hover-bg);
}
.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
