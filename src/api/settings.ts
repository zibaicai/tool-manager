import { invoke } from '@tauri-apps/api/core';

export interface CustomBg {
  path: string;
  /** light / dark */
  theme: string;
}

export interface ThemeSettings {
  /** default / light / dark */
  mode: string;
  /** 页面透明度 0 ~ 1.0 */
  opacity: number;
  /** 弹窗背景不透明度 0 ~ 1.0 */
  dialogOpacity: number;
  /** 弹窗背景模糊半径 px（0 = 不模糊） */
  dialogBlur: number;
  /** "builtin:light-1" 或自定义图片绝对路径 */
  bg: string | null;
  customBgs: CustomBg[];
}

export function loadThemeSettings(): Promise<ThemeSettings> {
  return invoke('load_theme_settings');
}

export function saveThemeSettings(settings: ThemeSettings): Promise<void> {
  return invoke('save_theme_settings', { settings });
}
