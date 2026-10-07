<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import BaseDialog from '../common/BaseDialog.vue';
import { useCategoryStore, type CategoryGroup } from '../../stores/categories';
import { launchTool } from '../../api/launcher';
import { alert } from '../../composables/useFeedback';
import { fuzzySearch } from '../../utils/fuzzySearch';
import type { Tool } from '../../types';

/**
 * 全局搜索命令面板（Ctrl+K / 顶栏放大镜）：
 * 模糊匹配标题、描述与所属分组，结果按相关度降序、按分组穿插显示；
 * ↑↓ 选择、Enter 启动、Esc 关闭（BaseDialog 统一处理）。
 */
const emit = defineEmits<{ close: [] }>();
const store = useCategoryStore();

const query = ref('');
const activeIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

/** 工具 → 所属业务分组（复用方案 A 的归一化分组，避免 "(EXE)" 并列） */
function groupOf(tool: Tool): CategoryGroup | null {
  return store.sortedCategoryGroups.find((g) => g.ids.includes(tool.categoryId)) ?? null;
}

interface Row {
  tool: Tool;
  group: string;
  index: number;
}

const rows = computed<Row[]>(() =>
  fuzzySearch(query.value, store.allTools, [
    { text: (t) => t.title, weight: 1 },
    { text: (t) => t.desc ?? '', weight: 0.5 },
    { text: (t) => groupOf(t)?.name ?? '', weight: 0.4 },
  ]).map((r) => ({ tool: r.item, group: groupOf(r.item)?.name ?? '未分组', index: 0 })),
);
// 填充渲染用的全局序号（与显示顺序一致，供键盘导航）
rows.value.forEach((r, i) => (r.index = i));

/** 相邻同组合并显示组头 */
const grouped = computed(() => {
  const out: { name: string; rows: Row[] }[] = [];
  for (const r of rows.value) {
    const last = out[out.length - 1];
    if (last && last.name === r.group) last.rows.push(r);
    else out.push({ name: r.group, rows: [r] });
  }
  return out;
});

watch(query, () => (activeIndex.value = 0));

onMounted(() => nextTick(() => inputRef.value?.focus()));

function move(delta: number) {
  if (!rows.value.length) return;
  activeIndex.value = (activeIndex.value + delta + rows.value.length) % rows.value.length;
  nextTick(() =>
    document
      .querySelector(`[data-row="${activeIndex.value}"]`)
      ?.scrollIntoView({ block: 'nearest' }),
  );
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    move(1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    move(-1);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    void launchAt(activeIndex.value);
  }
}

async function launchAt(index: number) {
  const row = rows.value[index];
  if (!row) return;
  emit('close');
  try {
    await launchTool(row.tool);
  } catch (e) {
    await alert('启动失败: ' + e);
  }
}

function locate(row: Row) {
  store.setActiveCategory(row.tool.categoryId);
  emit('close');
}
</script>

<template>
  <BaseDialog title="搜索工具" width="min(600px, 92vw)" top="12vh" @close="emit('close')">
    <input
      ref="inputRef"
      v-model="query"
      class="tm-text-input search-input"
      placeholder="搜索工具名、描述或分组…"
      spellcheck="false"
      @keydown="onKeydown"
    />

    <div v-if="!query.trim()" class="empty">
      输入关键词即可搜索，支持模糊匹配（打错一两个字符也能找到）
    </div>
    <div v-else-if="!rows.length" class="empty">未找到相关工具</div>
    <div v-else class="result-list">
      <div v-for="g in grouped" :key="g.name" class="group">
        <div class="group-name">{{ g.name }}</div>
        <div
          v-for="r in g.rows"
          :key="r.tool.id"
          class="row"
          :class="{ active: r.index === activeIndex }"
          :data-row="r.index"
          @mouseenter="activeIndex = r.index"
          @click="launchAt(r.index)"
        >
          <div class="info">
            <div class="tool-title" :title="r.tool.title">{{ r.tool.title }}</div>
            <div v-if="r.tool.desc" class="tool-desc" :title="r.tool.desc">{{ r.tool.desc }}</div>
          </div>
          <button class="mini" title="跳转到该工具所在分组" @click.stop="locate(r)">定位</button>
        </div>
      </div>
      <div class="foot-hint">↑↓ 选择 · Enter 启动 · Esc 关闭</div>
    </div>
  </BaseDialog>
</template>

<style scoped>
.search-input {
  font-size: 14px;
  padding: 9px 12px;
}
.empty {
  padding: 28px 0;
  text-align: center;
  font-size: 13px;
  color: var(--text-sub, #888);
}
.result-list {
  margin-top: 10px;
  max-height: 46vh;
  overflow-y: auto;
}
.group-name {
  font-size: 12px;
  color: var(--text-sub, #888);
  padding: 8px 4px 4px;
  user-select: none;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 8px;
  border-radius: 6px;
  cursor: pointer;
}
.row.active {
  background: var(--dialog-hover, rgba(255, 255, 255, 0.08));
}
.info {
  flex: 1;
  min-width: 0;
}
.tool-title {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tool-desc {
  font-size: 12px;
  color: var(--text-sub, #888);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}
.mini {
  flex-shrink: 0;
  padding: 3px 10px;
  font-size: 12px;
  border: 1px solid var(--dialog-border, #444);
  background: transparent;
  color: var(--text-sub, #aaa);
  border-radius: 5px;
}
.mini:hover {
  color: var(--primary, #2f8cff);
  border-color: var(--primary, #2f8cff);
}
.foot-hint {
  padding: 8px 4px 0;
  font-size: 11px;
  color: var(--text-sub, #888);
  text-align: center;
  user-select: none;
}
</style>
