import { getCurrentWindow, type Window } from '@tauri-apps/api/window';

/** 缩放方向（与 Tauri 的 ResizeDirection 字符串保持一致；该类型在当前版本未导出） */
export type ResizeDirection =
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'East'
  | 'West';

// 纯浏览器调试环境（无 Tauri 注入）下降级为空操作，避免报错
const hasTauri = '__TAURI_INTERNALS__' in window;

export const appWindow: Window | null = hasTauri ? getCurrentWindow() : null;

export async function minimize(): Promise<void> {
  await appWindow?.minimize();
}

export async function toggleMaximize(): Promise<void> {
  await appWindow?.toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  await appWindow?.close();
}

export async function isMaximized(): Promise<boolean> {
  return (await appWindow?.isMaximized()) ?? false;
}

/** 从指定方向开始边缘缩放（无边框窗口 8 向热区用） */
export async function startResize(direction: ResizeDirection): Promise<void> {
  await appWindow?.startResizeDragging(direction);
}

/** 监听最大化/还原状态变化，返回取消监听函数 */
export async function onMaximizeChange(
  cb: (maximized: boolean) => void,
): Promise<() => void> {
  if (!appWindow) return () => {};
  const win = appWindow;
  const unlisten = await win.onResized(async () => {
    cb(await win.isMaximized());
  });
  return unlisten;
}
