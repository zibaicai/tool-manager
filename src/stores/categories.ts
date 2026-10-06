import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Category, Tool } from '../types';
import type { CategoryType } from '../constants';
import { CATEGORY_TYPES, DEFAULT_WEIGHT } from '../constants';
import { addCategory, deleteCategory, loadMenuConfig, renameCategory, setScanRoot } from '../api/config';
import { useCmdToolsStore } from './cmdTools';
import { useExeToolsStore } from './exeTools';

/**
 * 展示层分组：把按 "(EXE)" 后缀命名的手动分类与同名扫描分类合并为一个业务分类。
 * 纯视图概念，不回写 menu.json；activeCategoryId 仍指向真实分类 id。
 */
export interface CategoryGroup {
  /** 归一化名称（去空白、小写）作稳定 key */
  key: string;
  /** 展示名（归一化后，去 "(EXE)" 后缀） */
  name: string;
  /** 主分类 id：scan 成员优先（承载编辑/权重/scanPath），否则取排序最前者 */
  primaryId: string;
  /** 组内全部真实分类 id（用于工具过滤） */
  ids: string[];
  /** 组内手动录入分类 id（作为新增 EXE 的落点），无则 null */
  manualId: string | null;
}

/** 去掉名称尾部的 "(EXE)"/"（EXE）" 或独立 "EXE" 后缀，得到业务名 */
function normalizeCatName(name: string): string {
  return name
    .replace(/[\s　]*[（(]\s*exe\s*[）)]\s*$/i, '')
    .replace(/[\s　]+exe\s*$/i, '')
    .trim();
}

/** 分组 key：归一化后去除全部空白并小写（"Web 扫描" 与 "Web扫描(EXE)" 同组） */
function groupKey(name: string): string {
  return normalizeCatName(name).replace(/\s+/g, '').toLowerCase();
}

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

  /** 按归一化名称分组后的展示列表 */
  const sortedCategoryGroups = computed<CategoryGroup[]>(() => {
    const map = new Map<string, CategoryGroup>();
    // 先按权重降序排列原始分类，确保组内顺序稳定且 scan 排在前面
    const ordered = [...categories.value].sort(
      (a, b) => (b.weight ?? DEFAULT_WEIGHT) - (a.weight ?? DEFAULT_WEIGHT),
    );
    for (const c of ordered) {
      const key = groupKey(c.name);
      const g = map.get(key);
      if (g) {
        g.ids.push(c.id);
        if (c.type === CATEGORY_TYPES.MANUAL && !g.manualId) {
          g.manualId = c.id;
        }
      } else {
        map.set(key, {
          key,
          name: normalizeCatName(c.name),
          primaryId: c.type === CATEGORY_TYPES.SCAN ? c.id : c.id, // 默认先放；scan 会覆盖
          ids: [c.id],
          manualId: c.type === CATEGORY_TYPES.MANUAL ? c.id : null,
        });
      }
    }
    // scan 分类优先作为主 id
    for (const c of ordered) {
      if (c.type === CATEGORY_TYPES.SCAN) {
        const key = groupKey(c.name);
        const g = map.get(key);
        if (g) g.primaryId = c.id;
      }
    }
    return Array.from(map.values());
  });

  /** 当前激活的分类组 */
  const activeGroup = computed<CategoryGroup | null>(() => {
    if (!activeCategoryId.value) return null;
    return (
      sortedCategoryGroups.value.find((g) => g.ids.includes(activeCategoryId.value)) ?? null
    );
  });

  /** 当前激活组下的全部工具（合并同组 scan + manual） */
  const activeTools = computed(() => {
    const ids = activeGroup.value?.ids ?? [activeCategoryId.value].filter(Boolean);
    return allTools.value.filter((t) => ids.includes(t.categoryId));
  });

  /** 侧边栏展示顺序：权重降序（越大越靠前），同权重保持配置文件原顺序（依赖稳定排序） */
  const sortedCategories = computed(() =>
    [...categories.value].sort(
      (a, b) => (b.weight ?? DEFAULT_WEIGHT) - (a.weight ?? DEFAULT_WEIGHT),
    ),
  );

  function toolsOf(categoryId: string): Tool[] {
    return allTools.value.filter((t) => t.categoryId === categoryId);
  }

  function toolsOfGroup(group: CategoryGroup): Tool[] {
    return allTools.value.filter((t) => group.ids.includes(t.categoryId));
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
        activeCategoryId.value = sortedCategoryGroups.value[0]?.primaryId ?? '';
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

  /** 新增 EXE 的落点分类 id：组内有 manual 分类用 manual，否则用主分类 */
  const exeTargetCategoryId = computed(
    () => activeGroup.value?.manualId ?? activeGroup.value?.primaryId ?? '',
  );

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
    sortedCategoryGroups,
    activeGroup,
    exeTargetCategoryId,
    toolsOf,
    toolsOfGroup,
    init,
    refresh,
    addCat,
    renameCat,
    deleteCat,
    setActiveCategory,
    saveScanRoot,
  };
});
