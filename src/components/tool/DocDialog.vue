<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import type { Tool } from '../../types';
import { readTextFile } from '../../api/fs';
import { renderMarkdown } from '../../utils/markdown';

const props = defineProps<{ tool: Tool }>();
const emit = defineEmits<{ close: [] }>();

const loading = ref(true);
const error = ref<string | null>(null);
const html = ref('');

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown);
  document.body.style.overflow = 'hidden';
  if (!props.tool.docPath) {
    loading.value = false;
    error.value = '该工具未提供 Ops.md';
    return;
  }
  try {
    const md = await readTextFile(props.tool.docPath);
    html.value = renderMarkdown(md);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  document.body.style.overflow = '';
});
</script>

<template>
  <Teleport to="body">
    <div class="overlay" @click.self="emit('close')">
      <div class="panel" role="dialog" aria-modal="true">
      <header class="panel-header">
        <span class="doc-title">{{ tool.title }} · 使用文档</span>
        <button class="close-btn" title="关闭 (Esc)" @click="emit('close')">✕</button>
      </header>

      <div class="panel-body">
        <div v-if="loading" class="state">文档加载中...</div>
        <div v-else-if="error" class="state error">{{ error }}</div>
        <!-- eslint-disable-next-line vue/no-v-html -->
        <article v-else class="markdown-body" v-html="html"></article>
      </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  width: 100%;
  height: 100vh;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  justify-content: flex-start;
  z-index: 1000;
}
.panel {
  background: #fff;
  --text: #1f2329;
  --text-sub: #6b7280;
  width: min(880px, 94vw);
  height: 100vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 24px rgba(0, 0, 0, 0.18);
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--card-border);
  flex-shrink: 0;
}
.doc-title {
  font-size: 15px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 16px;
  color: var(--text-sub);
  padding: 4px 8px;
  border-radius: 4px;
  flex-shrink: 0;
}
.close-btn:hover {
  background: #f0f1f3;
  color: var(--text);
}
.panel-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.state {
  padding: 60px 20px;
  text-align: center;
  color: var(--text-sub);
}
.state.error {
  color: #d33;
}

/* ---- 精简 markdown 排版 ---- */
.markdown-body {
  padding: 24px 32px 48px;
  font-size: 14px;
  line-height: 1.7;
  color: var(--text);
  word-wrap: break-word;
}
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 20px 0 10px;
  font-weight: 600;
  line-height: 1.35;
}
.markdown-body :deep(h1) {
  font-size: 22px;
  padding-bottom: 8px;
  border-bottom: 1px solid #eaecef;
}
.markdown-body :deep(h2) {
  font-size: 18px;
  padding-bottom: 6px;
  border-bottom: 1px solid #eaecef;
}
.markdown-body :deep(h3) {
  font-size: 16px;
}
.markdown-body :deep(h4) {
  font-size: 14px;
}
.markdown-body :deep(p),
.markdown-body :deep(ul),
.markdown-body :deep(ol),
.markdown-body :deep(blockquote),
.markdown-body :deep(table),
.markdown-body :deep(pre) {
  margin: 10px 0;
}
.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 24px;
}
.markdown-body :deep(li) {
  margin: 4px 0;
}
.markdown-body :deep(a) {
  color: var(--primary);
  text-decoration: none;
}
.markdown-body :deep(a:hover) {
  text-decoration: underline;
}
.markdown-body :deep(code) {
  font-family: Consolas, 'Courier New', monospace;
  font-size: 13px;
  background: #f3f4f6;
  padding: 2px 5px;
  border-radius: 4px;
}
.markdown-body :deep(pre) {
  background: #f6f8fa;
  padding: 12px 14px;
  border-radius: 6px;
  overflow-x: auto;
}
.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  font-size: 13px;
  line-height: 1.5;
}
.markdown-body :deep(blockquote) {
  padding: 4px 14px;
  border-left: 3px solid #d1d5db;
  color: var(--text-sub);
}
.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  display: block;
  overflow-x: auto;
}
.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid #d1d5db;
  padding: 6px 12px;
  text-align: left;
}
.markdown-body :deep(th) {
  background: #f6f8fa;
  font-weight: 600;
}
.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 20px 0;
}
.markdown-body :deep(img) {
  max-width: 100%;
}
</style>
