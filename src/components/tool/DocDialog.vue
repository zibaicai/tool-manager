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
  background: var(--dialog-bg);
  backdrop-filter: blur(var(--dialog-blur));
  -webkit-backdrop-filter: blur(var(--dialog-blur));
  --text: var(--dialog-fg);
  --text-sub: var(--dialog-fg-sub);
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
  border-bottom: 1px solid var(--dialog-border);
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
  background: var(--dialog-hover);
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
  border-bottom: 1px solid var(--dialog-border);
}
.markdown-body :deep(h2) {
  font-size: 18px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--dialog-border);
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
  background: var(--dialog-code-bg);
  padding: 2px 5px;
  border-radius: 4px;
}
.markdown-body :deep(pre) {
  background: var(--dialog-code-block-bg);
  padding: 12px 14px;
  border-radius: 6px;
  overflow-x: auto;
}
.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  font-size: 13px;
  line-height: 1.55;
}

/* ---- highlight.js 语法高亮：token 配色全部来自 --code-* 主题变量 ----
   只按官方 token class 着单 span 的颜色，不做通杀覆盖，保证高亮层级不丢失 */
.markdown-body :deep(.hljs) {
  color: var(--code-fg);
  font-family: Consolas, 'Courier New', monospace;
}
.markdown-body :deep(.hljs-comment),
.markdown-body :deep(.hljs-quote) {
  color: var(--code-comment);
  font-style: italic;
}
.markdown-body :deep(.hljs-keyword),
.markdown-body :deep(.hljs-selector-tag),
.markdown-body :deep(.hljs-doctag) {
  color: var(--code-keyword);
}
.markdown-body :deep(.hljs-string),
.markdown-body :deep(.hljs-regexp) {
  color: var(--code-string);
}
.markdown-body :deep(.hljs-number),
.markdown-body :deep(.hljs-literal) {
  color: var(--code-number);
}
.markdown-body :deep(.hljs-title),
.markdown-body :deep(.hljs-section) {
  color: var(--code-title);
}
.markdown-body :deep(.hljs-built_in),
.markdown-body :deep(.hljs-type) {
  color: var(--code-builtin);
}
.markdown-body :deep(.hljs-name),
.markdown-body :deep(.hljs-tag) {
  color: var(--code-tag);
}
.markdown-body :deep(.hljs-attr),
.markdown-body :deep(.hljs-attribute),
.markdown-body :deep(.hljs-selector-class),
.markdown-body :deep(.hljs-selector-id) {
  color: var(--code-attr);
}
.markdown-body :deep(.hljs-meta),
.markdown-body :deep(.hljs-meta .hljs-keyword) {
  color: var(--code-meta);
}
.markdown-body :deep(.hljs-symbol),
.markdown-body :deep(.hljs-bullet) {
  color: var(--code-symbol);
}
.markdown-body :deep(.hljs-variable),
.markdown-body :deep(.hljs-template-variable),
.markdown-body :deep(.hljs-selector-attr) {
  color: var(--code-variable);
}
.markdown-body :deep(.hljs-addition) {
  color: var(--code-addition);
  background-color: var(--code-addition-bg);
  border-radius: 3px;
}
.markdown-body :deep(.hljs-deletion) {
  color: var(--code-deletion);
  background-color: var(--code-deletion-bg);
  border-radius: 3px;
}
.markdown-body :deep(.hljs-emphasis) {
  font-style: italic;
}
.markdown-body :deep(.hljs-strong) {
  font-weight: 600;
}
.markdown-body :deep(blockquote) {
  padding: 4px 14px;
  border-left: 3px solid var(--input-border);
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
  border: 1px solid var(--input-border);
  padding: 6px 12px;
  text-align: left;
}
.markdown-body :deep(th) {
  background: var(--dialog-code-block-bg);
  font-weight: 600;
}
.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--dialog-border);
  margin: 20px 0;
}
.markdown-body :deep(img) {
  max-width: 100%;
}
</style>
