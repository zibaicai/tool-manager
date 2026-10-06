import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Tool } from '../types';
import { PROGRAM_EXTENSIONS } from '../constants';

export function loadExeTools(): Promise<Tool[]> {
  return invoke('load_exe_tools');
}

export function addExeTool(
  exePath: string,
  categoryId: string,
  title?: string,
  desc?: string,
  admin?: boolean,
  stopPath?: string,
  stopAdmin?: boolean,
  weight = 0,
): Promise<Tool> {
  return invoke('add_exe_tool', {
    exePath,
    categoryId,
    title: title?.trim() || null,
    desc: desc?.trim() || null,
    admin: admin ?? false,
    stopPath: stopPath?.trim() || null,
    stopAdmin: stopAdmin ?? false,
    weight: weight || null,
  });
}

/** 更新已录入工具的标题/副标题/管理员启动/关闭脚本/权重（标题传空串表示清除；weight 传 0 表示默认） */
export function updateExeTool(
  id: string,
  title: string,
  desc: string,
  admin: boolean,
  stopPath: string,
  stopAdmin: boolean,
  weight = 0,
): Promise<Tool> {
  return invoke('update_exe_tool', {
    id,
    title: title.trim() || null,
    desc: desc.trim() || null,
    admin,
    stopPath: stopPath.trim() || null,
    stopAdmin,
    weight: weight || null,
  });
}

export function removeExeTool(id: string): Promise<void> {
  return invoke('remove_exe_tool', { id });
}

/** 弹出系统文件选择框，返回所选 exe/bat/cmd 的绝对路径（取消时返回 null） */
export async function pickExe(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [{ name: '可执行文件', extensions: [...PROGRAM_EXTENSIONS] }],
  });
  return typeof selected === 'string' ? selected : null;
}
