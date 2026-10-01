import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { loadThemeSettings, saveThemeSettings, type CustomBg } from '../api/settings';

/** 每个主题大类的内置背景图（public/themes 下，随应用打包） */
export const BUILTIN_BGS: Record<'light' | 'dark', string[]> = {
  light: ['light-1', 'light-2', 'light-3'],
  dark: ['dark-1', 'dark-2', 'dark-3'],
};

/** 背景引用转可加载 URL："builtin:xxx" → 打包资源；绝对路径 → asset 协议 */
export function bgUrl(bg: string): string {
  return bg.startsWith('builtin:') ? `/themes/${bg.slice(8)}.svg` : convertFileSrc(bg);
}

export const useSettingsStore = defineStore('settings', () => {
  const mode = ref<'default' | 'light' | 'dark'>('default');
  const opacity = ref(0.85);
  const bg = ref<string | null>(null);
  const customBgs = ref<CustomBg[]>([]);

  const themed = computed(() => mode.value !== 'default');

  /** 把当前设置应用到 DOM（root data-theme + 透明度变量 + body 背景图） */
  function apply() {
    const root = document.documentElement;
    const body = document.body;
    if (mode.value === 'default') {
      root.removeAttribute('data-theme');
      body.style.backgroundImage = '';
      return;
    }
    root.dataset.theme = mode.value;
    const o = Math.min(1, Math.max(0, opacity.value));
    root.style.setProperty('--page-alpha', String(o));
    root.style.setProperty('--card-alpha', String(Math.min(1, o + 0.15)));
    root.style.setProperty('--btn-alpha', String(Math.min(1, o + 0.25)));
    body.style.backgroundImage = bg.value ? `url("${bgUrl(bg.value)}")` : '';
    body.style.backgroundSize = 'cover';
    body.style.backgroundPosition = 'center';
    body.style.backgroundRepeat = 'no-repeat';
    body.style.backgroundAttachment = 'fixed';
  }

  async function persist() {
    await saveThemeSettings({
      mode: mode.value,
      opacity: opacity.value,
      bg: bg.value,
      customBgs: customBgs.value,
    });
  }

  async function init() {
    try {
      const s = await loadThemeSettings();
      mode.value = (['default', 'light', 'dark'].includes(s.mode) ? s.mode : 'default') as
        | 'default'
        | 'light'
        | 'dark';
      opacity.value = s.opacity;
      bg.value = s.bg;
      customBgs.value = s.customBgs ?? [];
    } catch {
      // 读取失败保持默认（未设置主题 = 目前效果）
    }
    apply();
  }

  /** 切换主题大类；切到主题模式且当前背景不属于该类时，自动选该类第一张内置图 */
  async function setMode(m: 'default' | 'light' | 'dark') {
    mode.value = m;
    if (m !== 'default') {
      const validBuiltin = bg.value?.startsWith(`builtin:${m}-`);
      const validCustom = customBgs.value.some((c) => c.theme === m && c.path === bg.value);
      if (!validBuiltin && !validCustom) bg.value = `builtin:${BUILTIN_BGS[m][0]}`;
    }
    apply();
    await persist();
  }

  async function setOpacity(v: number) {
    opacity.value = v;
    apply();
    await persist();
  }

  async function selectBg(ref2: string) {
    bg.value = ref2;
    apply();
    await persist();
  }

  /** 从本地导入图片到当前主题大类的背景列表，并立即选中 */
  async function importBg(): Promise<string | null> {
    if (mode.value === 'default') return null;
    const picked = await open({
      title: '选择背景图片',
      multiple: false,
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'svg'] }],
    });
    if (!picked || Array.isArray(picked)) return null;
    if (!customBgs.value.some((c) => c.path === picked)) {
      customBgs.value.push({ path: picked, theme: mode.value });
    }
    bg.value = picked;
    apply();
    await persist();
    return picked;
  }

  /** 移除自定义背景；若正在使用则回退到该类第一张内置图 */
  async function removeCustomBg(path: string) {
    const entry = customBgs.value.find((c) => c.path === path);
    customBgs.value = customBgs.value.filter((c) => c.path !== path);
    if (bg.value === path) {
      bg.value = mode.value !== 'default' ? `builtin:${BUILTIN_BGS[mode.value][0]}` : null;
    }
    void entry;
    apply();
    await persist();
  }

  return {
    mode,
    opacity,
    bg,
    customBgs,
    themed,
    init,
    setMode,
    setOpacity,
    selectBg,
    importBg,
    removeCustomBg,
  };
});
