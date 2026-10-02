// Commitlint 配置：继承 Conventional Commits 规范
// 提交格式：<type>(scope?): <subject>
// 例：feat(launcher): 新增关闭工具按钮、fix: 修复窗口边框问题
/** @type {import('@commitlint/types').UserConfig} */
export default {
  extends: ['@commitlint/config-conventional'],
};
