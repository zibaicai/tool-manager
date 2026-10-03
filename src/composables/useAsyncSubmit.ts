import { ref } from 'vue';

/**
 * 弹窗提交动作的统一状态机：submitting 标志 + error 文本 + try/catch/finally。
 *
 * run 返回是否执行成功（true=成功，调用方可据此关闭弹窗）；
 * action 抛错时错误信息（String(e)）写入 error，不改变提交前的表单值。
 */
export function useAsyncSubmit() {
  const submitting = ref(false);
  const error = ref('');

  async function run(action: () => Promise<void>): Promise<boolean> {
    error.value = '';
    submitting.value = true;
    try {
      await action();
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      submitting.value = false;
    }
  }

  return { submitting, error, run };
}
