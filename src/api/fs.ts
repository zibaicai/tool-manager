import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Tool } from '../types';

/** 一次性扫描全部 CMD 工具分类（生效扫描根由后端按菜单统一解析，前端不再逐分类调用） */
export function scanAllCmdTools(): Promise<Tool[]> {
  return invoke('scan_all_cmd_tools');
}

export function readTextFile(path: string): Promise<string> {
  return invoke('read_text_file', { path });
}

/** 更新 CMD 工具的标题/副标题/权重/脚本执行目录/启动模式/启动命令/环境变量
 * （传空串表示清除，标题恢复自动派生；weight 传 0 表示默认） */
export function updateCmdTool(
  tool: Tool,
  title: string,
  desc: string,
  weight = 0,
  execDir = '',
  launchMode = '',
  launchCommand = '',
  env: Record<string, string> | null = null,
): Promise<Tool> {
  return invoke('update_cmd_tool', {
    id: tool.id,
    path: tool.path,
    categoryId: tool.categoryId,
    title: title.trim() || null,
    desc: desc.trim() || null,
    weight: weight || null,
    execDir: execDir.trim() || null,
    launchMode: launchMode.trim() || null,
    launchCommand: launchCommand.trim() || null,
    env: env && Object.keys(env).length > 0 ? env : null,
  });
}

/** 手动分配 CMD 工具到指定目录（分类）；categoryId 传 null 恢复自动归属 */
export function assignCmdTool(tool: Tool, categoryId: string | null): Promise<Tool> {
  return invoke('assign_cmd_tool', { id: tool.id, path: tool.path, categoryId });
}

/** 注册外部目录为 CMD 工具（如天狐工具箱等第三方工具源），分配到指定分类。
 *  目录需含 .exe/.bat/.ps1/.py 等可执行文件 */
export function registerCmdTool(
  path: string,
  categoryId: string,
  weight = 0,
): Promise<Tool> {
  return invoke('register_cmd_tool', {
    path,
    categoryId,
    weight: weight || null,
  });
}

/** 移除手动注册的外部 CMD 工具（只删记录，不动磁盘目录）；扫描产物会被后端拒绝 */
export function removeCmdTool(tool: Tool): Promise<void> {
  return invoke('remove_cmd_tool', { id: tool.id, path: tool.path });
}

/** 弹出系统目录选择框，返回所选目录的绝对路径（取消时返回 null） */
export async function pickDir(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === 'string' ? selected : null;
}
