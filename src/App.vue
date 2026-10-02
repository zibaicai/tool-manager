<script setup lang="ts">
import { onMounted, ref } from 'vue';
import TitleBar from './components/layout/TitleBar.vue';
import SideBar from './components/layout/SideBar.vue';
import ContentArea from './components/layout/ContentArea.vue';
import SettingsDialog from './components/layout/SettingsDialog.vue';
import { useToolsStore } from './stores/tools';
import { useSettingsStore } from './stores/settings';

const store = useToolsStore();
const settings = useSettingsStore();
const showSettings = ref(false);

onMounted(() => {
  settings.init();
  store.init();
});
</script>

<template>
  <div class="app">
    <TitleBar @settings="showSettings = true" />
    <div class="body">
      <SideBar />
      <ContentArea />
    </div>
    <SettingsDialog v-if="showSettings" @close="showSettings = false" />
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
