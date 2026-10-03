<script setup lang="ts">
import { computed, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useCategoryStore } from '../../stores/categories';
import { CATEGORY_TYPES, DEFAULT_WEIGHT, type CategoryType } from '../../constants';
import { useAsyncSubmit } from '../../composables/useAsyncSubmit';
import BaseDialog from '../common/BaseDialog.vue';

const props = defineProps<{ mode: 'add' | 'rename' }>();
const emit = defineEmits<{ close: [] }>();
const store = useCategoryStore();

const activeCat = store.categories.find((c) => c.id === store.activeCategoryId);
const name = ref(props.mode === 'rename' ? (activeCat?.name ?? '') : '');
const catType = ref<CategoryType>(
  props.mode === 'rename'
    ? (activeCat?.type === CATEGORY_TYPES.SCAN ? CATEGORY_TYPES.SCAN : CATEGORY_TYPES.MANUAL)
    : CATEGORY_TYPES.MANUAL,
);
const weight = ref<number>(
  props.mode === 'rename' ? (activeCat?.weight ?? DEFAULT_WEIGHT) : DEFAULT_WEIGHT,
);
/** 分类级扫描目录；仅 scan 类型编辑时展示，空串表示回退顶层 scanRoot */
const scanPath = ref(props.mode === 'rename' ? (activeCat?.scanPath ?? '') : '');
const showScanPath = computed(
  () => props.mode === 'rename' && catType.value === CATEGORY_TYPES.SCAN,
);
const { submitting, error, run } = useAsyncSubmit();

/** 调出系统目录选择框 */
async function browse() {
  const selected = await open({
    title: '选择该分类的扫描目录（留空则使用顶层 scanRoot）',
    directory: true,
    multiple: false,
    defaultPath: scanPath.value || undefined,
  });
  if (typeof selected === 'string') scanPath.value = selected;
}

async function submit() {
  const w = Number.isFinite(weight.value) ? (weight.value as number) : DEFAULT_WEIGHT;
  const ok = await run(async () => {
    if (props.mode === 'add') {
      await store.addCat(name.value, catType.value, w);
    } else {
      await store.renameCat(store.activeCategoryId, name.value, w, scanPath.value);
    }
  });
  if (ok) emit('close');
}
</script>

<template>
  <BaseDialog
    :title="mode === 'add' ? '添加目录' : `编辑「${activeCat?.name ?? ''}」`"
    width="min(440px, 92vw)"
    @close="emit('close')"
  >
    <label class="tm-field-label">目录名称</label>
    <input
      v-model="name"
      class="tm-text-input"
      type="text"
      placeholder="左侧菜单显示的名称"
      @keyup.enter="submit"
    />

    <label class="tm-field-label">排序权重</label>
    <input
      v-model.number="weight"
      class="tm-text-input"
      type="number"
      step="1"
      placeholder="0"
      @keyup.enter="submit"
    />
    <p class="tm-hint">数值越大目录越靠上；权重相同的目录按创建先后排列，默认 0</p>

    <template v-if="mode === 'add'">
      <label class="tm-field-label">目录类型</label>
      <select v-model="catType" class="tm-text-input">
        <option :value="CATEGORY_TYPES.MANUAL">EXE 手动录入</option>
        <option :value="CATEGORY_TYPES.SCAN">CMD 自动扫描（扫描 scanRoot 汇总目录）</option>
      </select>
      <p class="tm-hint">新建的 CMD 目录初始为空，请在工具卡片的编辑菜单中通过「所属目录」分配工具</p>
    </template>

    <template v-else-if="showScanPath">
      <label class="tm-field-label">分类专属扫描目录（可选）</label>
      <div class="tm-path-row">
        <input
          v-model="scanPath"
          class="tm-text-input"
          type="text"
          placeholder="留空则使用全局 scanRoot"
          spellcheck="false"
          @keyup.enter="submit"
        />
        <button class="tm-browse-btn" type="button" title="浏览选择目录" @click="browse">
          浏览…
        </button>
      </div>
      <p class="tm-hint">
        设置后该分类只扫描此目录（按 dirs 名单/孤儿规则）；清空保存则回退到全局 scanRoot
      </p>
    </template>

    <p v-if="error" class="tm-error">{{ error }}</p>

    <template #footer>
      <button class="tm-btn" @click="emit('close')">取消</button>
      <button class="tm-btn primary" :disabled="submitting" @click="submit">
        {{ submitting ? '保存中...' : '保存' }}
      </button>
    </template>
  </BaseDialog>
</template>
