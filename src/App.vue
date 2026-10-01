<script setup lang="ts">
import { onMounted } from 'vue';
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
    <SideBar />
    <ContentArea />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  background: var(--bg);
}
</style>
