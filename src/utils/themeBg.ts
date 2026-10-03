import { convertFileSrc } from '@tauri-apps/api/core';
import { BUILTIN_BG_PREFIX, THEME_MODES, type ThemeMode } from '../constants';

/** 每个主题大类的内置背景图（public/themes 下，随应用打包） */
export const BUILTIN_BGS: Record<Exclude<ThemeMode, typeof THEME_MODES.DEFAULT>, string[]> = {
  [THEME_MODES.LIGHT]: ['light-1', 'light-2', 'light-3'],
  [THEME_MODES.DARK]: ['dark-1', 'dark-2', 'dark-3'],
};

/** 背景引用转可加载 URL："builtin:xxx" → 打包资源；绝对路径 → asset 协议 */
export function bgUrl(bg: string): string {
  return bg.startsWith(BUILTIN_BG_PREFIX)
    ? `/themes/${bg.slice(BUILTIN_BG_PREFIX.length)}.svg`
    : convertFileSrc(bg);
}
