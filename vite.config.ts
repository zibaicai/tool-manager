import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// Tauri 期望前端固定运行在 1420 端口
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
