import { ref } from 'vue';

/**
 * 全局反馈通道：替代原生 window.confirm/alert，
 * 由 FeedbackHost 统一渲染为带主题（透明度/模糊）的 BaseDialog。
 *
 * confirm/alert 均返回 Promise；多个请求排队依次展示。
 */
export interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  /** 确认按钮使用红色危险样式 */
  danger?: boolean;
}

interface PendingRequest {
  kind: 'confirm' | 'alert';
  title: string;
  message: string;
  confirmText: string;
  cancelText: string;
  danger: boolean;
  resolve: (value: boolean) => void;
}

const queue = ref<PendingRequest[]>([]);
/** 当前正在展示的请求（队列首项），供 FeedbackHost 渲染 */
const current = ref<PendingRequest | null>(null);

function enqueue(req: Omit<PendingRequest, 'resolve'>): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    queue.value.push({ ...req, resolve });
    if (!current.value) current.value = queue.value[0];
  });
}

function settle(value: boolean) {
  const req = current.value;
  if (!req) return;
  req.resolve(value);
  queue.value.shift();
  current.value = queue.value[0] ?? null;
}

/** 确认弹窗：确认返回 true，取消/点遮罩/按 Esc 返回 false */
export function confirm(options: ConfirmOptions): Promise<boolean> {
  return enqueue({
    kind: 'confirm',
    title: options.title ?? '请确认',
    message: options.message,
    confirmText: options.confirmText ?? '确定',
    cancelText: options.cancelText ?? '取消',
    danger: options.danger ?? false,
  });
}

/** 提示弹窗（单按钮），关闭（含 Esc/遮罩）即 resolve */
export function alert(message: string, title = '提示'): Promise<void> {
  return enqueue({
    kind: 'alert',
    title,
    message,
    confirmText: '知道了',
    cancelText: '取消',
    danger: false,
  }).then(() => undefined);
}

export function useFeedback() {
  return { current, settle, confirm, alert };
}
