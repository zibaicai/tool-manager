import {
  BTN_ALPHA_STEP,
  CARD_ALPHA_STEP,
  DIALOG_BLUR_MAX,
  OPACITY_MAX,
  OPACITY_MIN,
  THEME_MODES,
  type ThemeMode,
} from '../constants';
import { bgUrl } from './themeBg';

/** DOM 注入所需的设置快照（settings store 状态的纯数据视图） */
export interface ThemeDomState {
  mode: ThemeMode;
  opacity: number;
  dialogOpacity: number;
  dialogBlur: number;
  bg: string | null;
}

/** 弹窗参数与主题模式无关，默认/浅色/深色下都注入到根节点 */
function applyDialogVars(root: HTMLElement, state: ThemeDomState) {
  const a = Math.min(OPACITY_MAX, Math.max(OPACITY_MIN, state.dialogOpacity));
  const b = Math.min(DIALOG_BLUR_MAX, Math.max(OPACITY_MIN, state.dialogBlur));
  root.style.setProperty('--dialog-alpha', String(a));
  root.style.setProperty('--dialog-blur', `${b}px`);
}

/**
 * 把主题设置应用到 DOM：root data-theme + 透明度变量 + body 背景图。
 * 纯副作用函数，不读写 store，便于独立调用与测试。
 */
export function applyThemeDom(state: ThemeDomState) {
  const root = document.documentElement;
  const body = document.body;
  applyDialogVars(root, state);
  if (state.mode === THEME_MODES.DEFAULT) {
    root.removeAttribute('data-theme');
    body.style.backgroundImage = '';
    return;
  }
  root.dataset.theme = state.mode;
  const o = Math.min(OPACITY_MAX, Math.max(OPACITY_MIN, state.opacity));
  root.style.setProperty('--page-alpha', String(o));
  root.style.setProperty('--card-alpha', String(Math.min(OPACITY_MAX, o + CARD_ALPHA_STEP)));
  root.style.setProperty('--btn-alpha', String(Math.min(OPACITY_MAX, o + BTN_ALPHA_STEP)));
  body.style.backgroundImage = state.bg ? `url("${bgUrl(state.bg)}")` : '';
  body.style.backgroundSize = 'cover';
  body.style.backgroundPosition = 'center';
  body.style.backgroundRepeat = 'no-repeat';
  body.style.backgroundAttachment = 'fixed';
}
