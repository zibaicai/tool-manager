<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useToolsStore } from '../../stores/tools';

const emit = defineEmits<{ close: [] }>();
const store = useToolsStore();

const scanRoot = ref(store.scanRoot);
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
  error.value = '';
  submitting.value = true;
  try {
    await store.saveScanRoot(scanRoot.value);
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
          <span>设置</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <label class="field-label">CMD 工具扫描根目录（scanRoot）</label>
          <div class="path-row">
            <input
              v-model="scanRoot"
              class="text-input"
              type="text"
              placeholder="例如 D:\ScriptingTool\cmd_tool"
              spellcheck="false"
              @keyup.enter="save"
            />
            <button class="browse-btn" type="button" title="浏览选择目录" @click="browse">浏览…</button>
          </div>
          <p class="hint">
            CMD 类工具统一存放的汇总目录；未单独配置扫描目录的分类都会在此目录下扫描。
            保存后自动重新扫描，清空后保存可取消该设置。
          </p>
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <footer class="dialog-footer">
          <button class="btn" @click="emit('close')">取消</button>
          <button class="btn primary" :disabled="submitting" @click="save">
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
  width: min(520px, 94vw);
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
  color: var(--text);
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
  margin-top: 0;
}
.path-row {
  display: flex;
  gap: 8px;
}
.text-input {
  flex: 1;
  min-width: 0;
  box-sizing: border-box;
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  outline: none;
  background: var(--input-bg);
}
.text-input:focus {
  border-color: var(--primary);
}
.browse-btn {
  flex-shrink: 0;
  padding: 7px 14px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
  color: var(--text);
}
.browse-btn:hover {
  background: var(--dialog-btn-hover);
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
  border-top: 1px solid var(--dialog-border);
}
.btn {
  padding: 7px 18px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
  color: var(--text);
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
