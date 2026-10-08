<script setup lang="ts">
import { ref } from 'vue';
import { useCategoryStore } from '../../stores/categories';
import ToolGrid from '../tool/ToolGrid.vue';
import ExeToolForm from '../tool/ExeToolForm.vue';
import CmdToolForm from '../tool/CmdToolForm.vue';

const store = useCategoryStore();
const showExeForm = ref(false);
const showCmdForm = ref(false);
</script>

<template>
  <main class="content">
    <header class="header">
      <h2>{{ store.activeGroup?.name || '' }}</h2>
      <span class="count">{{ store.activeTools.length }} 个工具</span>
      <button v-if="store.activeGroup" class="add-btn" @click="showCmdForm = true">＋ 添加 CMD 工具</button>
      <button v-if="store.activeGroup" class="add-btn" @click="showExeForm = true">＋ 添加 EXE 工具</button>
    </header>

    <div v-if="store.loading" class="state">加载中...</div>
    <div v-else-if="store.error" class="state error">{{ store.error }}</div>
    <div v-else-if="store.activeTools.length === 0" class="state">
      该分类暂无工具，可点击右上角添加 CMD / EXE 工具
    </div>
    <ToolGrid v-else :tools="store.activeTools" />

    <ExeToolForm v-if="showExeForm" @close="showExeForm = false" />
    <CmdToolForm v-if="showCmdForm" @close="showCmdForm = false" />
  </main>
</template>

<style scoped>
.content {
  flex: 1;
  padding: 20px 24px;
  overflow-y: auto;
}
.header {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 16px;
}
.header h2 {
  font-size: 18px;
  font-weight: 600;
}
.count {
  font-size: 12px;
  color: var(--text-sub);
}
.add-btn {
  margin-left: auto;
  padding: 6px 14px;
  font-size: 13px;
  background: var(--btn-primary-bg, var(--primary));
  color: #fff;
  border: none;
  border-radius: 6px;
}
.add-btn:hover {
  filter: brightness(1.08);
}
.state {
  padding: 40px;
  text-align: center;
  color: var(--text-sub);
}
.state.error {
  color: #d33;
}
</style>
