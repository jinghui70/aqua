<script setup lang="ts">
// json Tab: json-ui 兼容格式预览 + 复制/下载(无配置)。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useTauri } from "@/composables/useTauri";
import { useProjectStore } from "@/stores/project";
import { downloadText } from "@/composables/useDownload";

const props = defineProps<{ tableCode: string }>();

const tauri = useTauri();
const store = useProjectStore();
const preview = ref("");

async function refresh() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateFrontendJson(
      store.currentProject,
      props.tableCode
    );
  } catch {
    /* 已提示 */
  }
}

watch(() => props.tableCode, refresh, { immediate: true });

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  downloadText(`${props.tableCode}.json`, preview.value);
}
</script>

<template>
  <div class="flex flex-col gap-12">
    <div class="flex items-center gap-8">
      <span class="text-13 text-gray-500">json-ui 兼容格式</span>
      <div class="flex-1" />
      <el-button size="small" @click="copy">复制</el-button>
      <el-button size="small" type="primary" @click="download">下载</el-button>
    </div>
    <el-input
      v-model="preview"
      type="textarea"
      :rows="22"
      readonly
      class="font-mono"
    />
  </div>
</template>
