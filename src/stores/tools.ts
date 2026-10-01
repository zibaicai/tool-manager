import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Category, Tool } from '../types';
import { addCategory, deleteCategory, loadMenuConfig, renameCategory } from '../api/config';
import { scanCmdTools, updateCmdTool, assignCmdTool } from '../api/fs';
import { addExeTool, loadExeTools, removeExeTool, updateExeTool } from '../api/exe';

export const useToolsStore = defineStore('tools', () => {
  const categories = ref<Category[]>([]);
  const tools = ref<Tool[]>([]);
  const activeCategoryId = ref<string>('');
  const scanRoot = ref<string>('');
  const loading = ref(false);
  const error = ref<string | null>(null);

  const activeTools = computed(() =>
    tools.value.filter((t) => t.categoryId === activeCategoryId.value),
  );

  async function init() {
    activeCategoryId.value = '';
    await refresh();
  }

  /** 重新读取 menu.json 并扫描；当前分类仍存在时保留选中状态 */
  async function refresh() {
    loading.value = true;
    error.value = null;
    try {
      const menu = await loadMenuConfig();
      categories.value = menu.categories;
      scanRoot.value = menu.scanRoot ?? '';
      if (!categories.value.some((c) => c.id === activeCategoryId.value)) {
        activeCategoryId.value = categories.value[0]?.id ?? '';
      }
      await reloadAll();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function reloadAll() {
    const all: Tool[] = [];
    for (const cat of categories.value) {
      const root = cat.scanPath || scanRoot.value;
      if (cat.type === 'scan' && root) {
        try {
          const list = await scanCmdTools(root, cat.id, cat.dirs);
          all.push(...list);
        } catch (e) {
          console.warn(`扫描分类 ${cat.name} 失败:`, e);
        }
      }
      // manual / system 类型后续阶段补
    }
    // EXE 工具为全局手动录入，条目自带 categoryId
    try {
      all.push(...(await loadExeTools()));
    } catch (e) {
      console.warn('加载 EXE 工具失败:', e);
    }
    tools.value = all;
  }

  /** 录入 EXE 工具；成功后写入列表 */
  async function addExe(
    exePath: string,
    title?: string,
    desc?: string,
    admin?: boolean,
    stopPath?: string,
  ): Promise<void> {
    const tool = await addExeTool(exePath, activeCategoryId.value, title, desc, admin, stopPath);
    const idx = tools.value.findIndex((t) => t.id === tool.id);
    if (idx >= 0) tools.value[idx] = tool;
    else tools.value.push(tool);
  }

  /** 更新 EXE 工具标题/副标题/管理员启动/关闭脚本 */
  async function updateExe(
    id: string,
    title: string,
    desc: string,
    admin: boolean,
    stopPath: string,
  ): Promise<void> {
    const tool = await updateExeTool(id, title, desc, admin, stopPath);
    const idx = tools.value.findIndex((t) => t.id === id);
    if (idx >= 0) tools.value[idx] = tool;
  }

  /** 更新 CMD 工具标题/副标题 */
  async function updateCmd(id: string, title: string, desc: string): Promise<void> {
    const current = tools.value.find((t) => t.id === id);
    if (!current) return;
    const tool = await updateCmdTool(current, title, desc);
    const idx = tools.value.findIndex((t) => t.id === id);
    if (idx >= 0) tools.value[idx] = tool;
  }

  /** 手动分配 CMD 工具到指定目录；categoryId 为 null 恢复自动归属 */
  async function assignCmd(id: string, categoryId: string | null): Promise<void> {
    const current = tools.value.find((t) => t.id === id);
    if (!current) return;
    await assignCmdTool(current, categoryId);
    await refresh();
  }

  /** 移除 EXE 工具录入 */
  async function removeExe(id: string): Promise<void> {
    await removeExeTool(id);
    tools.value = tools.value.filter((t) => t.id !== id);
  }

  /** 新增目录并切换过去 */
  async function addCat(name: string, type: 'scan' | 'manual'): Promise<void> {
    const cat = await addCategory(name, type);
    await refresh();
    activeCategoryId.value = cat.id;
  }

  /** 重命名目录 */
  async function renameCat(id: string, name: string): Promise<void> {
    await renameCategory(id, name);
    await refresh();
  }

  /** 删除目录（refresh 会自动回退选中到第一个分类） */
  async function deleteCat(id: string): Promise<void> {
    await deleteCategory(id);
    await refresh();
  }

  function setActiveCategory(id: string) {
    activeCategoryId.value = id;
  }

  return {
    categories,
    tools,
    activeCategoryId,
    activeTools,
    loading,
    error,
    init,
    refresh,
    reloadAll,
    addExe,
    updateExe,
    updateCmd,
    assignCmd,
    removeExe,
    addCat,
    renameCat,
    deleteCat,
    setActiveCategory,
  };
});
