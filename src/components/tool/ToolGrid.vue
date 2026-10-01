<script setup lang="ts">
import type { Tool } from '../../types';
import ToolCard from './ToolCard.vue';

defineProps<{ tools: Tool[] }>();
</script>

<template>
  <div class="grid">
    <ToolCard v-for="t in tools" :key="t.id" :tool="t" />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}
</style>
