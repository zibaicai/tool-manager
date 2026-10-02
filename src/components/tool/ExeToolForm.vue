<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useToolsStore } from '../../stores/tools';
import { pickExe } from '../../api/exe';

const emit = defineEmits<{ close: [] }>();
const store = useToolsStore();

const categoryName = computed(
  () => store.categories.find((c) => c.id === store.activeCategoryId)?.name ?? '当前分类',
);

const exePath = ref('');
const title = ref('');
const desc = ref('');
const admin = ref(false);
const stopPath = ref('');
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

async function browse() {
  try {
    const picked = await pickExe();
    if (picked) exePath.value = picked;
  } catch (e) {
    error.value = '打开文件选择框失败: ' + e;
  }
}

async function submit() {
  error.value = '';
  if (!exePath.value.trim()) {
    error.value = '请填写或选择 exe 的绝对路径';
    return;
  }
  submitting.value = true;
  try {
    await store.addExe(exePath.value.trim(), title.value, desc.value, admin.value, stopPath.value);
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
          <span>添加 EXE 工具到「{{ categoryName }}」</span>
          <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
        </header>

        <div class="dialog-body">
          <label class="field-label">exe 绝对路径或启动命令</label>
          <div class="path-row">
            <input
              v-model="exePath"
              class="path-input"
              type="text"
              placeholder='D:\tools\app.exe -c "参数" 或 D:\tools\start-svc.bat'
              @keyup.enter="submit"
            />
            <button class="browse-btn" @click="browse">浏览...</button>
          </div>
          <p class="hint">
            支持 .exe 与 .bat（启动服务脚本），可带启动参数，如 D:\Program\Nmap\zenmap\bin\pythonw.exe -c
            "from zenmapGUI.App import run;run()"（路径含空格时可加英文引号）
          </p>

          <label class="field-label">工具标题（可选）</label>
          <input
            v-model="title"
            class="path-input"
            type="text"
            placeholder="留空自动取 exe 同目录 README.md 的 H1，无则用文件名"
          />

          <label class="field-label">副标题 · 用途描述（可选）</label>
          <input
            v-model="desc"
            class="path-input"
            type="text"
            placeholder="一句话说明工具用途，展示在标题下方"
            @keyup.enter="submit"
          />
          <p class="hint">Ops.md 放入 exe 同目录后，刷新即启用「使用文档」按钮</p>

          <label class="check-row">
            <input v-model="admin" type="checkbox" />
            <span>以管理员身份运行（启动时弹出 UAC 授权，如 net.exe 启动系统服务）</span>
          </label>

          <label class="field-label">关闭脚本 · .bat（可选）</label>
          <input
            v-model="stopPath"
            class="path-input"
            type="text"
            placeholder="D:\tools\stop-svc.bat，填写后卡片出现「关闭工具」按钮"
          />
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <footer class="dialog-footer">
          <button class="btn" @click="emit('close')">取消</button>
          <button class="btn primary" :disabled="submitting" @click="submit">
            {{ submitting ? '添加中...' : '添加' }}
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
  width: min(560px, 92vw);
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
.path-row {
  display: flex;
  gap: 8px;
}
.path-input {
  flex: 1;
  min-width: 0;
  padding: 7px 10px;
  font-size: 13px;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  outline: none;
}
.path-input:focus {
  border-color: var(--primary);
}
.browse-btn {
  flex-shrink: 0;
  padding: 7px 14px;
  font-size: 13px;
  border: 1px solid var(--dialog-border);
  background: var(--dialog-btn-bg);
  border-radius: 6px;
}
.browse-btn:hover {
  background: var(--dialog-btn-hover);
}
.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-sub);
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
