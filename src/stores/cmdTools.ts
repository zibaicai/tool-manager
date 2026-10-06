import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Tool } from '../types';
import { assignCmdTool, scanAllCmdTools, updateCmdTool } from '../api/fs';

/** CMD 自动扫描工具领域：扫描结果列表与覆盖项（标题/手动分配）写入 */
export const useCmdToolsStore = defineStore('cmdTools', () => {
  const tools = ref<Tool[]>([]);

  /** 一次性扫描全部分类（生效根由后端解析） */
  async function reload() {
    try {
      tools.value = await scanAllCmdTools();
    } catch (e) {
      console.warn('扫描 CMD 工具失败:', e);
      tools.value = [];
    }
  }

  /** 更新 CMD 工具标题/副标题/权重/脚本执行目录（weight 默认 0，execDir 空 = 工具目录本身） */
  async function updateCmd(
    id: string,
    title: string,
    desc: string,
    weight = 0,
    execDir = '',
  ): Promise<void> {
    const current = tools.value.find((t) => t.id === id);
    if (!current) return;
    const tool = await updateCmdTool(current, title, desc, weight, execDir);
    const idx = tools.value.findIndex((t) => t.id === id);
    if (idx >= 0) tools.value[idx] = tool;
  }

  /** 手动分配 CMD 工具到指定目录；categoryId 为 null 恢复自动归属。
   *  归属变更后需由调用方触发 categoryStore.refresh() 重新分区扫描 */
  async function assignCmd(id: string, categoryId: string | null): Promise<void> {
    const current = tools.value.find((t) => t.id === id);
    if (!current) return;
    await assignCmdTool(current, categoryId);
  }

  return { tools, reload, updateCmd, assignCmd };
});
