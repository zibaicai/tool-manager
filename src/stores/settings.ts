import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { loadThemeSettings, saveThemeSettings, type CustomBg } from '../api/settings';
import {
  BTN_ALPHA_STEP,
  BUILTIN_BG_PREFIX,
  CARD_ALPHA_STEP,
  DEFAULT_DIALOG_BLUR,
  DEFAULT_DIALOG_OPACITY,
  DEFAULT_PAGE_OPACITY,
  DIALOG_BLUR_MAX,
  OPACITY_MAX,
  OPACITY_MIN,
  THEME_MODES,
  type ThemeMode,
} from '../constants';

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

export const useSettingsStore = defineStore('settings', () => {
  const mode = ref<ThemeMode>(THEME_MODES.DEFAULT);
  const opacity = ref(DEFAULT_PAGE_OPACITY);
  const dialogOpacity = ref(DEFAULT_DIALOG_OPACITY);
  const dialogBlur = ref(DEFAULT_DIALOG_BLUR);
  const bg = ref<string | null>(null);
  const customBgs = ref<CustomBg[]>([]);

  const themed = computed(() => mode.value !== THEME_MODES.DEFAULT);

  /** 弹窗参数与主题模式无关，默认/浅色/深色下都注入到根节点 */
  function applyDialogVars(root: HTMLElement) {
    const a = Math.min(OPACITY_MAX, Math.max(OPACITY_MIN, dialogOpacity.value));
    const b = Math.min(DIALOG_BLUR_MAX, Math.max(OPACITY_MIN, dialogBlur.value));
    root.style.setProperty('--dialog-alpha', String(a));
    root.style.setProperty('--dialog-blur', `${b}px`);
  }

  /** 把当前设置应用到 DOM（root data-theme + 透明度变量 + body 背景图） */
  function apply() {
    const root = document.documentElement;
    const body = document.body;
    applyDialogVars(root);
    if (mode.value === THEME_MODES.DEFAULT) {
      root.removeAttribute('data-theme');
      body.style.backgroundImage = '';
      return;
    }
    root.dataset.theme = mode.value;
    const o = Math.min(OPACITY_MAX, Math.max(OPACITY_MIN, opacity.value));
    root.style.setProperty('--page-alpha', String(o));
    root.style.setProperty('--card-alpha', String(Math.min(OPACITY_MAX, o + CARD_ALPHA_STEP)));
    root.style.setProperty('--btn-alpha', String(Math.min(OPACITY_MAX, o + BTN_ALPHA_STEP)));
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
      dialogOpacity: dialogOpacity.value,
      dialogBlur: dialogBlur.value,
      bg: bg.value,
      customBgs: customBgs.value,
    });
  }

  async function init() {
    try {
      const s = await loadThemeSettings();
      mode.value = (Object.values(THEME_MODES).includes(s.mode as ThemeMode)
        ? s.mode
        : THEME_MODES.DEFAULT) as ThemeMode;
      opacity.value = s.opacity;
      dialogOpacity.value = s.dialogOpacity ?? DEFAULT_DIALOG_OPACITY;
      dialogBlur.value = s.dialogBlur ?? DEFAULT_DIALOG_BLUR;
      bg.value = s.bg;
      customBgs.value = s.customBgs ?? [];
    } catch {
      // 读取失败保持默认（未设置主题 = 目前效果）
    }
    apply();
  }

  /** 切换主题大类；切到主题模式且当前背景不属于该类时，自动选该类第一张内置图 */
  async function setMode(m: ThemeMode) {
    mode.value = m;
    if (m !== THEME_MODES.DEFAULT) {
      const validBuiltin = bg.value?.startsWith(`${BUILTIN_BG_PREFIX}${m}-`);
      const validCustom = customBgs.value.some((c) => c.theme === m && c.path === bg.value);
      if (!validBuiltin && !validCustom) bg.value = `${BUILTIN_BG_PREFIX}${BUILTIN_BGS[m][0]}`;
    }
    apply();
    await persist();
  }

  async function setOpacity(v: number) {
    opacity.value = v;
    apply();
    await persist();
  }

  async function setDialogOpacity(v: number) {
    dialogOpacity.value = v;
    apply();
    await persist();
  }

  async function setDialogBlur(v: number) {
    dialogBlur.value = v;
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
    if (mode.value === THEME_MODES.DEFAULT) return null;
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
      bg.value =
        mode.value !== THEME_MODES.DEFAULT
          ? `${BUILTIN_BG_PREFIX}${BUILTIN_BGS[mode.value][0]}`
          : null;
    }
    void entry;
    apply();
    await persist();
  }

  return {
    mode,
    opacity,
    dialogOpacity,
    dialogBlur,
    bg,
    customBgs,
    themed,
    init,
    setMode,
    setOpacity,
    setDialogOpacity,
    setDialogBlur,
    selectBg,
    importBg,
    removeCustomBg,
  };
});
