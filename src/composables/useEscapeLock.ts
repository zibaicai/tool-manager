import { onBeforeUnmount, onMounted } from 'vue';

/**
 * 弹窗/抽屉的统一键盘与滚动锁定：
 * - Esc 触发 onClose
 * - 挂载期间锁定 body 滚动，卸载时还原
 *
 * 仅做这两件所有弹窗完全一致的事；遮罩点击关闭由 BaseDialog 处理。
 */
export function useEscapeLock(onClose: () => void) {
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown);
    document.body.style.overflow = 'hidden';
  });

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown);
    document.body.style.overflow = '';
  });
}
