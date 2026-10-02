<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import type { Tool } from '../../types';
import { useToolsStore } from '../../stores/tools';
import { CATEGORY_TYPES, TOOL_TYPES } from '../../constants';

const props = defineProps<{ tool: Tool }>();
const emit = defineEmits<{ close: [] }>();
const store = useToolsStore();

const title = ref(props.tool.title);
const desc = ref(props.tool.desc ?? '');
const admin = ref(props.tool.admin ?? false);
const stopPath = ref(props.tool.stopPath ?? '');
const assignedCat = ref(props.tool.categoryId);
const scanCats = computed(() =>
  store.sortedCategories.filter((c) => c.type === CATEGORY_TYPES.SCAN),
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

async function submit() {
  error.value = '';
  submitting.value = true;
  try {
    if (props.tool.type === TOOL_TYPES.EXE) {
      await store.updateExe(props.tool.id, title.value, desc.value, admin.value, stopPath.value);
    } else {
      await store.updateCmd(props.tool.id, title.value, desc.value);
      if (assignedCat.value !== props.tool.categoryId) {
        await store.assignCmd(props.tool.id, assignedCat.value || null);
      }
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
          <span>编辑「{{ tool.title }}」</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <label class="field-label">工具标题</label>
          <input
            v-model="title"
            class="text-input"
            type="text"
            placeholder="留空恢复自动取名"
          />

          <label class="field-label">副标题 · 用途描述</label>
          <input
            v-model="desc"
            class="text-input"
            type="text"
            placeholder="一句话说明工具用途，展示在标题下方"
            @keyup.enter="submit"
          />

          <label v-if="tool.type === TOOL_TYPES.EXE" class="check-row">
            <input v-model="admin" type="checkbox" />
            <span>以管理员身份运行（启动时弹出 UAC 授权）</span>
          </label>

          <template v-if="tool.type === TOOL_TYPES.EXE">
            <label class="field-label">关闭脚本 · .bat / .cmd（可选）</label>
            <input
              v-model="stopPath"
              class="text-input"
              type="text"
              placeholder="D:\tools\stop-svc.bat，填写后卡片出现「关闭工具」按钮"
            />
          </template>

          <template v-if="tool.type === TOOL_TYPES.CMD">
            <label class="field-label">所属目录</label>
            <select v-model="assignedCat" class="text-input">
              <option value="">自动（按目录扫描规则）</option>
              <option v-for="c in scanCats" :key="c.id" :value="c.id">{{ c.name }}</option>
            </select>
            <p class="hint">仅可选择 CMD 自动扫描类目录，且工具需位于目标目录的扫描根路径下</p>
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
  background: var(--dialog-bg);
  backdrop-filter: blur(var(--dialog-blur));
  -webkit-backdrop-filter: blur(var(--dialog-blur));
  --text: var(--dialog-fg);
  --text-sub: var(--dialog-fg-sub);
  width: min(480px, 92vw);
  border-radius: 8px;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.2);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  border-bottom: 1px solid var(--dialog-border);
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
  background: var(--dialog-hover);
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
  border: 1px solid var(--input-border);
  border-radius: 6px;
  outline: none;
}
.text-input:focus {
  border-color: var(--primary);
}
.error {
  margin-top: 8px;
  font-size: 12px;
  color: #d33;
}
.hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-sub);
  line-height: 1.5;
}
.check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 18px;
  border-top: 1px solid var(--dialog-border);
}
.btn {
  padding: 7px 18px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
}
.btn:hover:not(:disabled) {
  background: var(--dialog-btn-hover);
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
