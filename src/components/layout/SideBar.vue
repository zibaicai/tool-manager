<script setup lang="ts">
import { ref } from 'vue';
import { useCategoryStore } from '../../stores/categories';
import CategoryDialog from './CategoryDialog.vue';
import ConfirmDeleteDialog from './ConfirmDeleteDialog.vue';
import ThemeDialog from './ThemeDialog.vue';

const store = useCategoryStore();
const dialogMode = ref<'add' | 'rename' | null>(null);
const showDelete = ref(false);
const showTheme = ref(false);
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <span>Tool Manager</span>
      <button class="theme-btn" title="主题设置" @click="showTheme = true">🎨</button>
    </div>
    <nav class="menu">
      <div
        v-for="cat in store.sortedCategories"
        :key="cat.id"
        class="menu-item"
        :class="{ active: cat.id === store.activeCategoryId }"
        @click="store.setActiveCategory(cat.id)"
      >
        {{ cat.name }}
      </div>
    </nav>
    <div class="footer">
      <div class="cat-actions">
        <button class="icon-btn" title="添加目录" @click="dialogMode = 'add'">＋</button>
        <button
          class="icon-btn"
          title="编辑当前目录（名称 / 排序权重）"
          :disabled="!store.activeCategoryId"
          @click="dialogMode = 'rename'"
        >
          ✎
        </button>
        <button
          class="icon-btn danger"
          title="删除当前目录"
          :disabled="!store.activeCategoryId"
          @click="showDelete = true"
        >
          🗑
        </button>
      </div>
      <button class="refresh-btn" @click="store.refresh()">刷新</button>
    </div>
    <CategoryDialog v-if="dialogMode" :mode="dialogMode" @close="dialogMode = null" />
    <ConfirmDeleteDialog
      v-if="showDelete && store.activeCategoryId"
      :category-id="store.activeCategoryId"
      @close="showDelete = false"
    />
    <ThemeDialog v-if="showTheme" @close="showTheme = false" />
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  background: var(--sidebar-bg);
  color: var(--sidebar-fg);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.brand {
  padding: 18px 16px;
  font-size: 16px;
  font-weight: 600;
  color: var(--brand-fg, #fff);
  border-bottom: 1px solid #2a2f38;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.theme-btn {
  border: none;
  background: var(--icon-btn-bg, #262b34);
  border-radius: 6px;
  font-size: 14px;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.theme-btn:hover {
  background: var(--icon-btn-hover, #333a46);
}
.menu {
  flex: 1;
  min-height: 0;
  padding: 8px 0;
  overflow-y: auto;
}
.menu-item {
  padding: 10px 16px;
  font-size: 14px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.15s;
}
.menu-item:hover {
  background: var(--menu-hover, #262b34);
}
.menu-item.active {
  background: var(--menu-active-bg, #262b34);
  color: var(--brand-fg, #fff);
  border-left-color: var(--sidebar-active);
}
.footer {
  padding: 12px;
  border-top: 1px solid #2a2f38;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.cat-actions {
  display: flex;
  gap: 6px;
}
.icon-btn {
  width: 30px;
  height: 30px;
  background: var(--icon-btn-bg, #262b34);
  color: var(--sidebar-fg);
  border: none;
  border-radius: 6px;
  font-size: 14px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.icon-btn:hover:not(:disabled) {
  background: var(--icon-btn-hover, #333a46);
  color: var(--brand-fg, #fff);
}
.icon-btn.danger:hover:not(:disabled) {
  background: #4a2a2e;
  color: #ff9d9d;
}
.icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.refresh-btn {
  padding: 8px 18px;
  background: var(--btn-primary-bg, #2f8cff);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 13px;
}
.refresh-btn:hover {
  background: var(--btn-primary-hover-bg);
}
</style>
