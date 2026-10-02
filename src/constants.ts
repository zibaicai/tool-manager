/**
 * 全局约定常量：类型字符串、扩展名白名单、主题默认值与范围。
 * 与 Rust 端 src-tauri/src/constants.rs（及 launcher.rs 的扩展名常量）各持一份，
 * 跨语言无法共享定义——修改任一侧必须同步另一侧。
 */

// ---------- 工具类型（Tool.type 传输值，对应后端 TOOL_CMD / TOOL_EXE） ----------

export const TOOL_TYPES = { CMD: 'cmd', EXE: 'exe' } as const;
export type ToolType = (typeof TOOL_TYPES)[keyof typeof TOOL_TYPES];

// ---------- 分类类型（menu.json 的 type 落盘值，对应后端 CAT_SCAN / CAT_MANUAL） ----------

export const CATEGORY_TYPES = { SCAN: 'scan', MANUAL: 'manual' } as const;
export type CategoryType = (typeof CATEGORY_TYPES)[keyof typeof CATEGORY_TYPES];

// ---------- 主题模式 ----------

export const THEME_MODES = {
  DEFAULT: 'default',
  LIGHT: 'light',
  DARK: 'dark',
} as const;
export type ThemeMode = (typeof THEME_MODES)[keyof typeof THEME_MODES];

// ---------- 可执行扩展名白名单（镜像后端 launcher.rs SCRIPT/PROGRAM_EXTENSIONS） ----------

export const SCRIPT_EXTENSIONS = ['bat', 'cmd'] as const;
export const PROGRAM_EXTENSIONS = ['exe', 'bat', 'cmd'] as const;

// ---------- 内置背景图引用前缀 ----------

export const BUILTIN_BG_PREFIX = 'builtin:';

// ---------- 主题数值默认值与范围（镜像后端 constants.rs） ----------

/** 页面透明度默认值 */
export const DEFAULT_PAGE_OPACITY = 0.85;
/** 弹窗不透明度默认值 */
export const DEFAULT_DIALOG_OPACITY = 1;
/** 弹窗高斯模糊半径默认值（px） */
export const DEFAULT_DIALOG_BLUR = 0;
/** 不透明度下限/上限（持久化与运行时 clamp 共用） */
export const OPACITY_MIN = 0;
export const OPACITY_MAX = 1;
/** 弹窗模糊半径持久化上限（px），与后端 DIALOG_BLUR_MAX 对应；
 *  主题弹窗滑杆是更小的 UI 取值范围（DIALOG_BLUR_SLIDER_MAX），两者独立 */
export const DIALOG_BLUR_MAX = 60;
/** 主题弹窗模糊滑杆上限（纯 UI 取值范围） */
export const DIALOG_BLUR_SLIDER_MAX = 30;
/** 卡片/按钮透明度相对页面透明度的联动增量（仅前端展示规则） */
export const CARD_ALPHA_STEP = 0.15;
export const BTN_ALPHA_STEP = 0.25;
/** 目录排序权重默认值（与后端 DEFAULT_WEIGHT 对应） */
export const DEFAULT_WEIGHT = 0;
