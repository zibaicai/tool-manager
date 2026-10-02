<script setup lang="ts">
import { onMounted } from 'vue';
import TitleBar from './components/layout/TitleBar.vue';
import SideBar from './components/layout/SideBar.vue';
import ContentArea from './components/layout/ContentArea.vue';
import { useToolsStore } from './stores/tools';
import { useSettingsStore } from './stores/settings';

const store = useToolsStore();
const settings = useSettingsStore();
onMounted(() => {
  settings.init();
  store.init();
});
</script>

<template>
  <div class="app">
    <TitleBar />
    <div class="body">
      <SideBar />
      <ContentArea />
    </div>
  </div>
</template>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  /* 主题模式下为半透明蒙层，背景图（body）从标题栏到内容区整窗透出 */
  background: var(--bg);
}
.body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}
</style>
