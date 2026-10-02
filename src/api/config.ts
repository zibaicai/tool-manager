import { invoke } from '@tauri-apps/api/core';
import type { Category, MenuConfig } from '../types';

export function loadMenuConfig(): Promise<MenuConfig> {
  return invoke('load_menu_config');
}

/** 新增目录（分类）：scan=自动扫描，manual=EXE 手动录入；weight 为排序权重（越大越靠前） */
export function addCategory(
  name: string,
  categoryType: 'scan' | 'manual',
  weight: number,
): Promise<Category> {
  return invoke('add_category', { name, categoryType, weight });
}

/** 重命名目录并可同时调整权重（id 不变） */
export function renameCategory(id: string, name: string, weight: number): Promise<void> {
  return invoke('rename_category', { id, name, weight });
}

/** 删除目录（其下 EXE 录入保留在 exe-tools.json，但不再显示） */
export function deleteCategory(id: string): Promise<void> {
  return invoke('delete_category', { id });
}

/** 设置 CMD 工具统一扫描根目录（scanRoot）；传空串表示清除 */
export function setScanRoot(path: string): Promise<void> {
  const trimmed = path.trim();
  return invoke('set_scan_root', { path: trimmed || null });
}
