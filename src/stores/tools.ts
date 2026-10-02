import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Category, Tool } from '../types';
import type { CategoryType } from '../constants';
import { DEFAULT_WEIGHT } from '../constants';
import { addCategory, deleteCategory, loadMenuConfig, renameCategory, setScanRoot } from '../api/config';
import { scanAllCmdTools, updateCmdTool, assignCmdTool } from '../api/fs';
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

  /** 侧边栏展示顺序：权重降序（越大越靠前），同权重保持配置文件原顺序（依赖稳定排序） */
  const sortedCategories = computed(() =>
    [...categories.value].sort(
      (a, b) => (b.weight ?? DEFAULT_WEIGHT) - (a.weight ?? DEFAULT_WEIGHT),
    ),
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
        activeCategoryId.value = sortedCategories.value[0]?.id ?? '';
      }
      await reloadAll();
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 加载全部工具：CMD 扫描与 EXE 录入各一次后端调用，并行执行。
   *  生效扫描根（分类 scanPath → 顶层 scanRoot 的回退）只由后端解析，前端不再重复计算 */
  async function reloadAll() {
    const [cmdTools, exeTools] = await Promise.all([
      scanAllCmdTools().catch((e) => {
        console.warn('扫描 CMD 工具失败:', e);
        return [] as Tool[];
      }),
      loadExeTools().catch((e) => {
        console.warn('加载 EXE 工具失败:', e);
        return [] as Tool[];
      }),
    ]);
    tools.value = [...cmdTools, ...exeTools];
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

  /** 新增目录并切换过去；weight 为排序权重（越大越靠前，默认 0） */
  async function addCat(
    name: string,
    type: CategoryType,
    weight = DEFAULT_WEIGHT,
  ): Promise<void> {
    const cat = await addCategory(name, type, weight);
    await refresh();
    activeCategoryId.value = cat.id;
  }

  /** 重命名目录并可同时调整权重 */
  async function renameCat(id: string, name: string, weight: number): Promise<void> {
    await renameCategory(id, name, weight);
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

  /** 保存扫描根目录并按新路径重新扫描工具列表 */
  async function saveScanRoot(path: string): Promise<void> {
    const trimmed = path.trim();
    await setScanRoot(trimmed);
    scanRoot.value = trimmed;
    await reloadAll();
  }

  return {
    categories,
    sortedCategories,
    tools,
    activeCategoryId,
    activeTools,
    scanRoot,
    loading,
    error,
    init,
    refresh,
    reloadAll,
    saveScanRoot,
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
