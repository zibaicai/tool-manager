import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Tool } from '../types';
import { addExeTool, loadExeTools, removeExeTool, updateExeTool } from '../api/exe';

/** EXE 手动录入工具领域：列表状态与增删改；分类归属由调用方传入 */
export const useExeToolsStore = defineStore('exeTools', () => {
  const tools = ref<Tool[]>([]);

  async function reload() {
    try {
      tools.value = await loadExeTools();
    } catch (e) {
      console.warn('加载 EXE 工具失败:', e);
      tools.value = [];
    }
  }

  /** 录入 EXE 工具；成功后写入/替换列表 */
  async function addExe(
    exePath: string,
    categoryId: string,
    title?: string,
    desc?: string,
    admin?: boolean,
    stopPath?: string,
    stopAdmin?: boolean,
    weight = 0,
  ): Promise<void> {
    const tool = await addExeTool(
      exePath,
      categoryId,
      title,
      desc,
      admin,
      stopPath,
      stopAdmin,
      weight,
    );
    const idx = tools.value.findIndex((t) => t.id === tool.id);
    if (idx >= 0) tools.value[idx] = tool;
    else tools.value.push(tool);
  }

  /** 更新 EXE 工具标题/副标题/提权标记/关闭脚本/权重（weight 默认 0） */
  async function updateExe(
    id: string,
    title: string,
    desc: string,
    admin: boolean,
    stopPath: string,
    stopAdmin: boolean,
    weight = 0,
  ): Promise<void> {
    const tool = await updateExeTool(id, title, desc, admin, stopPath, stopAdmin, weight);
    const idx = tools.value.findIndex((t) => t.id === id);
    if (idx >= 0) tools.value[idx] = tool;
  }

  /** 移除 EXE 工具录入（仅删记录，不删 exe 文件） */
  async function removeExe(id: string): Promise<void> {
    await removeExeTool(id);
    tools.value = tools.value.filter((t) => t.id !== id);
  }

  return { tools, reload, addExe, updateExe, removeExe };
});
