import { invoke } from '@tauri-apps/api/core';
import type { CustomBg, ThemeSettings } from '../types';

export type { CustomBg, ThemeSettings };

export function loadThemeSettings(): Promise<ThemeSettings> {
  return invoke('load_theme_settings');
}

export function saveThemeSettings(settings: ThemeSettings): Promise<void> {
  return invoke('save_theme_settings', { settings });
}
