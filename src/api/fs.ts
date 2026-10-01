import { invoke } from '@tauri-apps/api/core';
import type { Tool } from '../types';

export function scanCmdTools(
  path: string,
  categoryId: string,
  dirs?: string[],
): Promise<Tool[]> {
  return invoke('scan_cmd_tools', { path, categoryId, dirs: dirs ?? null });
}

export function readTextFile(path: string): Promise<string> {
  return invoke('read_text_file', { path });
}

/** 更新 CMD 工具的标题/副标题（传空串表示清除，标题恢复自动派生） */
export function updateCmdTool(tool: Tool, title: string, desc: string): Promise<Tool> {
  return invoke('update_cmd_tool', {
    id: tool.id,
    path: tool.path,
    categoryId: tool.categoryId,
    title: title.trim() || null,
    desc: desc.trim() || null,
  });
}

/** 手动分配 CMD 工具到指定目录（分类）；categoryId 传 null 恢复自动归属 */
export function assignCmdTool(tool: Tool, categoryId: string | null): Promise<Tool> {
  return invoke('assign_cmd_tool', { id: tool.id, path: tool.path, categoryId });
}
