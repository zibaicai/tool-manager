import { invoke } from '@tauri-apps/api/core';
import type { Tool } from '../types';
import { TOOL_TYPES } from '../constants';

/**
 * 启动工具：
 * - cmd：在工具根目录打开一个新的命令行窗口
 * - exe：运行指定的 exe（path 为 exe 绝对路径，工作目录为其所在目录）
 */
export function launchTool(tool: Tool): Promise<void> {
  return invoke('launch_tool', {
    toolType: tool.type,
    path: tool.path,
    args: tool.args ?? null,
    admin: tool.admin ?? false,
  });
}

/** 在系统文件管理器中打开目录（exe 工具自动打开 exe 所在目录） */
export function openPath(path: string): Promise<void> {
  return invoke('open_path', { path });
}

/** 执行关闭脚本（.bat/.cmd）：与启动同一套逻辑（可见 cmd 窗口 / UAC 提权） */
export function stopTool(tool: Tool): Promise<void> {
  return invoke('launch_tool', {
    toolType: TOOL_TYPES.EXE,
    path: tool.stopPath,
    args: null,
    admin: tool.admin ?? false,
  });
}
