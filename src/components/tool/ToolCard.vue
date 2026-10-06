<script setup lang="ts">
import { ref } from 'vue';
import type { Tool } from '../../types';
import { launchTool, openPath, stopTool } from '../../api/launcher';
import { useExeToolsStore } from '../../stores/exeTools';
import { alert, confirm } from '../../composables/useFeedback';
import { TOOL_TYPES } from '../../constants';
import DocDialog from './DocDialog.vue';
import ExeToolEditDialog from './ExeToolEditDialog.vue';

const props = defineProps<{ tool: Tool }>();
const exeStore = useExeToolsStore();

const showDoc = ref(false);
const showEdit = ref(false);

function openDoc() {
  if (!props.tool.docPath) return;
  showDoc.value = true;
}

async function remove() {
  const ok = await confirm({
    title: '移除工具',
    message: `确定从列表移除「${props.tool.title}」吗？（不会删除 exe 文件本身）`,
    confirmText: '移除',
    danger: true,
  });
  if (!ok) return;
  try {
    await exeStore.removeExe(props.tool.id);
  } catch (e) {
    await alert('移除失败: ' + e);
  }
}

async function openCmd() {
  try {
    await launchTool(props.tool);
  } catch (e) {
    await alert('启动失败: ' + e);
  }
}

async function stop() {
  try {
    await stopTool(props.tool);
  } catch (e) {
    await alert('关闭失败: ' + e);
  }
}

async function openDir() {
  try {
    await openPath(props.tool.path);
  } catch (e) {
    await alert('打开目录失败: ' + e);
  }
}
</script>

<template>
  <div class="card">
    <div class="card-header">
      <div class="title" :title="tool.title">{{ tool.title }}</div>
      <div class="header-right">
        <button class="edit-btn" title="编辑标题/副标题" @click="showEdit = true">✎</button>
        <button v-if="tool.type === TOOL_TYPES.EXE" class="remove-btn" title="从列表移除" @click="remove">✕</button>
        <div class="type-tag" :class="tool.type">{{ tool.type.toUpperCase() }}</div>
      </div>
    </div>
    <div v-if="tool.desc" class="desc" :title="tool.desc">{{ tool.desc }}</div>
    <div class="actions">
      <button class="btn" :disabled="!tool.docPath" @click="openDoc">使用文档</button>
      <button class="btn primary" @click="openCmd">
        {{ tool.type === TOOL_TYPES.EXE ? '启动工具' : '命令窗口' }}
      </button>
      <button v-if="tool.stopPath" class="btn danger" title="执行关闭脚本停止服务" @click="stop">
        关闭工具
      </button>
      <button class="btn" @click="openDir">打开目录</button>
    </div>

    <DocDialog v-if="showDoc" :tool="tool" @close="showDoc = false" />
    <ExeToolEditDialog v-if="showEdit" :tool="tool" @close="showEdit = false" />
  </div>
</template>

<style scoped>
.card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: var(--radius);
  padding: 14px;
  box-shadow: var(--shadow);
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: transform 0.12s, box-shadow 0.12s;
}
.card:hover {
  transform: translateY(-2px);
  background: var(--card-hover-bg, var(--card-bg));
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.title {
  font-size: 15px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.type-tag {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: #eef4ff;
  color: var(--primary);
  flex-shrink: 0;
}
.type-tag.cmd {
  background: #eef4ff;
  color: #2f8cff;
}
.type-tag.exe {
  background: #fff2e8;
  color: #fa8c16;
}
.remove-btn,
.edit-btn {
  /* 默认隐藏，悬浮卡片时才显示；tag 在其右侧固定，避免位移与右侧空缺 */
  display: none;
  border: none;
  background: transparent;
  color: var(--text-sub);
  font-size: 12px;
  line-height: 1;
  padding: 3px 5px;
  border-radius: 4px;
}
.card:hover .remove-btn,
.card:hover .edit-btn {
  display: inline-block;
}
.remove-btn:hover {
  background: #fdeaea;
  color: #d33;
}
.edit-btn:hover {
  background: #eef4ff;
  color: var(--primary);
}
.desc {
  font-size: 12px;
  color: var(--text-sub);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.actions {
  display: flex;
  gap: 6px;
  margin-top: auto;
}
.btn {
  flex: 1;
  padding: 6px 0;
  font-size: 12px;
  border: 1px solid var(--card-border);
  background: var(--btn-surface, #fff);
  border-radius: 6px;
  color: var(--text);
  transition: background 0.12s;
}
.btn:hover:not(:disabled) {
  filter: brightness(1.06);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn.primary {
  background: var(--btn-primary-bg, var(--primary));
  color: var(--btn-primary-fg, #fff);
  border-color: transparent;
}
.btn.primary:hover {
  background: var(--btn-primary-hover-bg);
}
.btn.danger {
  background: var(--btn-danger-bg);
  color: var(--btn-danger-text);
  border-color: var(--btn-danger-border);
}
.btn.danger:hover {
  background: var(--btn-danger-hover-bg);
}
</style>
