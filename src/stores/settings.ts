import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { loadThemeSettings, saveThemeSettings, type CustomBg } from '../api/settings';
import {
  BUILTIN_BG_PREFIX,
  DEFAULT_DIALOG_BLUR,
  DEFAULT_DIALOG_OPACITY,
  DEFAULT_PAGE_OPACITY,
  THEME_MODES,
  type ThemeMode,
} from '../constants';
import { BUILTIN_BGS } from '../utils/themeBg';
import { applyThemeDom, type ThemeDomState } from '../utils/themeDom';

/**
 * 主题设置领域：仅负责状态与后端持久化。
 * DOM 注入见 utils/themeDom.ts，背景 URL 纯函数见 utils/themeBg.ts，
 * 系统文件选择框由 ThemeDialog 调起后把路径交给 addCustomBg。
 */
export const useSettingsStore = defineStore('settings', () => {
  const mode = ref<ThemeMode>(THEME_MODES.DEFAULT);
  const opacity = ref(DEFAULT_PAGE_OPACITY);
  const dialogOpacity = ref(DEFAULT_DIALOG_OPACITY);
  const dialogBlur = ref(DEFAULT_DIALOG_BLUR);
  const bg = ref<string | null>(null);
  const customBgs = ref<CustomBg[]>([]);

  const themed = computed(() => mode.value !== THEME_MODES.DEFAULT);

  function domState(): ThemeDomState {
    return {
      mode: mode.value,
      opacity: opacity.value,
      dialogOpacity: dialogOpacity.value,
      dialogBlur: dialogBlur.value,
      bg: bg.value,
    };
  }

  function apply() {
    applyThemeDom(domState());
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

  /** 注册并选中一张已从系统选择器挑好的背景图；默认外观下忽略 */
  async function addCustomBg(path: string) {
    if (mode.value === THEME_MODES.DEFAULT) return;
    if (!customBgs.value.some((c) => c.path === path)) {
      customBgs.value.push({ path, theme: mode.value });
    }
    bg.value = path;
    apply();
    await persist();
  }

  /** 移除自定义背景；若正在使用则回退到该类第一张内置图 */
  async function removeCustomBg(path: string) {
    customBgs.value = customBgs.value.filter((c) => c.path !== path);
    if (bg.value === path) {
      bg.value =
        mode.value !== THEME_MODES.DEFAULT
          ? `${BUILTIN_BG_PREFIX}${BUILTIN_BGS[mode.value][0]}`
          : null;
    }
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
    addCustomBg,
    removeCustomBg,
  };
});
