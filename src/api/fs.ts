import { invoke } from '@tauri-apps/api/core';
import type { Tool } from '../types';

/** 一次性扫描全部 CMD 工具分类（生效扫描根由后端按菜单统一解析，前端不再逐分类调用） */
export function scanAllCmdTools(): Promise<Tool[]> {
  return invoke('scan_all_cmd_tools');
}

export function readTextFile(path: string): Promise<string> {
  return invoke('read_text_file', { path });
}

/** 更新 CMD 工具的标题/副标题/权重（传空串表示清除，标题恢复自动派生；weight 传 0 表示默认） */
export function updateCmdTool(
  tool: Tool,
  title: string,
  desc: string,
  weight = 0,
): Promise<Tool> {
  return invoke('update_cmd_tool', {
    id: tool.id,
    path: tool.path,
    categoryId: tool.categoryId,
    title: title.trim() || null,
    desc: desc.trim() || null,
    weight: weight || null,
  });
}

/** 手动分配 CMD 工具到指定目录（分类）；categoryId 传 null 恢复自动归属 */
export function assignCmdTool(tool: Tool, categoryId: string | null): Promise<Tool> {
  return invoke('assign_cmd_tool', { id: tool.id, path: tool.path, categoryId });
}
