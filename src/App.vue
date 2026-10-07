<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import TitleBar from './components/layout/TitleBar.vue';
import SideBar from './components/layout/SideBar.vue';
import ContentArea from './components/layout/ContentArea.vue';
import SettingsDialog from './components/layout/SettingsDialog.vue';
import SearchPalette from './components/layout/SearchPalette.vue';
import FeedbackHost from './components/common/FeedbackHost.vue';
import WindowResizeEdges from './components/common/WindowResizeEdges.vue';
import { useCategoryStore } from './stores/categories';
import { useSettingsStore } from './stores/settings';

const store = useCategoryStore();
const settings = useSettingsStore();
const showSettings = ref(false);
const showSearch = ref(false);

/** Ctrl+K（macOS Cmd+K）唤起全局搜索面板 */
function onGlobalKey(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    showSearch.value = true;
  }
}

onMounted(() => {
  settings.init();
  store.init();
  window.addEventListener('keydown', onGlobalKey);
});

onBeforeUnmount(() => window.removeEventListener('keydown', onGlobalKey));
</script>

<template>
  <div class="app">
    <TitleBar @settings="showSettings = true" @search="showSearch = true" />
    <div class="body">
      <SideBar />
      <ContentArea />
    </div>
    <SettingsDialog v-if="showSettings" @close="showSettings = false" />
    <SearchPalette v-if="showSearch" @close="showSearch = false" />
    <FeedbackHost />
    <WindowResizeEdges />
  </div>
</template>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}
</style>
