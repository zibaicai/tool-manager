import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Category, Tool } from '../types';
import type { CategoryType } from '../constants';
import { DEFAULT_WEIGHT } from '../constants';
import { addCategory, deleteCategory, loadMenuConfig, renameCategory, setScanRoot } from '../api/config';
import { useCmdToolsStore } from './cmdTools';
import { useExeToolsStore } from './exeTools';

/**
 * 分类（menu.json）领域：目录列表/选中项/scanRoot 与刷新调度。
 * 工具列表本体分属 cmdTools / exeTools 两个 store，
 * 这里通过 computed 合并出视图层需要的 allTools / activeTools。
 */
export const useCategoryStore = defineStore('categories', () => {
  const categories = ref<Category[]>([]);
  const activeCategoryId = ref<string>('');
  const scanRoot = ref<string>('');
  const loading = ref(false);
  const error = ref<string | null>(null);

  const cmdStore = useCmdToolsStore();
  const exeStore = useExeToolsStore();

  /** CMD 扫描结果与 EXE 录入的合并视图 */
  const allTools = computed<Tool[]>(() => [...cmdStore.tools, ...exeStore.tools]);

  const activeTools = computed(() =>
    allTools.value.filter((t) => t.categoryId === activeCategoryId.value),
  );

  /** 侧边栏展示顺序：权重降序（越大越靠前），同权重保持配置文件原顺序（依赖稳定排序） */
  const sortedCategories = computed(() =>
    [...categories.value].sort(
      (a, b) => (b.weight ?? DEFAULT_WEIGHT) - (a.weight ?? DEFAULT_WEIGHT),
    ),
  );

  function toolsOf(categoryId: string): Tool[] {
    return allTools.value.filter((t) => t.categoryId === categoryId);
  }

  async function init() {
    activeCategoryId.value = '';
    await refresh();
  }

  /** 重新读取 menu.json 并并行重新扫描/加载两类工具；当前分类仍存在时保留选中状态 */
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
      await Promise.all([cmdStore.reload(), exeStore.reload()]);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
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

  /** 重命名目录并可同时调整权重与分类级扫描目录 */
  async function renameCat(
    id: string,
    name: string,
    weight: number,
    scanPath: string,
  ): Promise<void> {
    await renameCategory(id, name, weight, scanPath);
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
    await Promise.all([cmdStore.reload(), exeStore.reload()]);
  }

  return {
    categories,
    activeCategoryId,
    scanRoot,
    loading,
    error,
    allTools,
    activeTools,
    sortedCategories,
    toolsOf,
    init,
    refresh,
    addCat,
    renameCat,
    deleteCat,
    setActiveCategory,
    saveScanRoot,
  };
});
