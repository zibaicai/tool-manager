<script setup lang="ts">
import { ref, computed } from 'vue';
import { useCategoryStore } from '../../stores/categories';
import { useExeToolsStore } from '../../stores/exeTools';
import { pickExe } from '../../api/exe';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import BaseDialog from '../common/BaseDialog.vue';
import ExeToolFields from './ExeToolFields.vue';

const emit = defineEmits<{ close: [] }>();
const categoryStore = useCategoryStore();
const exeStore = useExeToolsStore();

const categoryName = computed(() => categoryStore.activeGroup?.name ?? '当前分类');

const exePath = ref('');
const title = ref('');
const desc = ref('');
const admin = ref(false);
const stopPath = ref('');
const stopAdmin = ref(false);
const { submitting, error, run } = useAsyncSubmit();

async function browse() {
  try {
    const picked = await pickExe();
    if (picked) exePath.value = picked;
  } catch (e) {
    error.value = '打开文件选择框失败: ' + e;
  }
}

async function submit() {
  if (!exePath.value.trim()) {
    error.value = '请填写或选择 exe 的绝对路径';
    return;
  }
  const ok = await run(() =>
    exeStore.addExe(
      exePath.value.trim(),
      categoryStore.exeTargetCategoryId,
      title.value,
      desc.value,
      admin.value,
      stopPath.value,
      stopAdmin.value,
    ),
  );
  if (ok) emit('close');
}
</script>

<template>
  <BaseDialog
    :title="`添加 EXE 工具到「${categoryName}」`"
    width="min(560px, 92vw)"
    @close="emit('close')"
  >
    <label class="tm-field-label">启动路径（可附带启动参数）</label>
    <div class="tm-path-row">
      <input
        v-model="exePath"
        class="tm-text-input"
        type="text"
        placeholder='D:\tools\app.exe -c "参数" 或 D:\tools\start-svc.bat'
        spellcheck="false"
        @keyup.enter="submit"
      />
      <button class="tm-browse-btn" @click="browse">浏览...</button>
    </div>
    <p class="tm-hint">
      支持 .exe / .bat / .cmd（bat/cmd 为启动服务脚本），可带启动参数，如 D:\Program\Nmap\zenmap\bin\pythonw.exe -c
      "from zenmapGUI.App import run;run()"（路径含空格时可加英文引号）
    </p>

    <ExeToolFields
      v-model:title="title"
      v-model:desc="desc"
      v-model:admin="admin"
      v-model:stop-path="stopPath"
      v-model:stop-admin="stopAdmin"
    />

    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="submit">
        {{ submitting ? '添加中...' : '添加' }}
      </button>
    </template>
  </BaseDialog>
</template>
