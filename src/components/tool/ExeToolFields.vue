<script setup lang="ts">
/**
 * EXE 工具的公共表单字段：标题 / 副标题 / 管理员启动 / 关闭脚本（含独立提权）。
 * 添加表单与编辑弹窗共用，保证字段与提示文案只有一份。
 * 启动路径（含浏览按钮）是添加场景独有内容，由父组件放在本组件之前。
 */
const title = defineModel<string>('title', { required: true });
const desc = defineModel<string>('desc', { required: true });
const admin = defineModel<boolean>('admin', { required: true });
const stopPath = defineModel<string>('stopPath', { required: true });
const stopAdmin = defineModel<boolean>('stopAdmin', { required: true });
</script>

<template>
  <label class="tm-field-label">自定义标题（可选）</label>
  <input v-model="title" class="tm-text-input" type="text" placeholder="留空则取 README.md 标题或文件名" />

  <label class="tm-field-label">副标题（可选）</label>
  <input v-model="desc" class="tm-text-input" type="text" placeholder="一句话说明工具用途" />

  <label class="tm-check-row">
    <input v-model="admin" type="checkbox" />
    <span>以管理员身份启动（启动时弹出 UAC 授权）</span>
  </label>

  <label class="tm-field-label">关闭脚本 · .bat / .cmd（可选）</label>
  <input
    v-model="stopPath"
    class="tm-text-input"
    type="text"
    placeholder="D:\tools\stop-svc.bat，填写后卡片出现「关闭工具」按钮"
  />
  <p class="tm-hint">仅在需要停止后台服务时填写；关闭脚本与启动命令分开执行</p>
  <label v-if="stopPath.trim()" class="tm-check-row">
    <input v-model="stopAdmin" type="checkbox" />
    <span>关闭脚本也以管理员身份运行（独立于启动提权设置）</span>
  </label>
</template>
