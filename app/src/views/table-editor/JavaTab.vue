<script setup lang="ts">
// java Tab: 配置(包名/类名/Lombok/注释)+ 实时预览 + 复制/下载。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useTauri } from "@/composables/useTauri";
import { useProjectStore } from "@/stores/project";
import { downloadText } from "@/composables/useDownload";

const props = defineProps<{ tableCode: string; active: boolean }>();

const tauri = useTauri();
const store = useProjectStore();

const useLombok = ref(true);
const includeComment = ref(true);
const packageName = ref("");
const className = ref("");
const preview = ref("");

async function refresh() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateJava(
      store.currentProject,
      props.tableCode,
      {
        useLombok: useLombok.value,
        includeComment: includeComment.value,
        package: packageName.value || undefined,
        className: className.value || undefined,
      }
    );
  } catch {
    /* 已提示 */
  }
}

// 配置变化实时刷新
watch([useLombok, includeComment, packageName, className, () => props.tableCode], refresh, {
  immediate: true,
});
// 切回本 tab 时重新生成,同步字段/索引的改动
watch(() => props.active, (a) => a && refresh());

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  const cls = className.value || props.tableCode;
  downloadText(`${cls}.java`, preview.value);
}
</script>

<template>
  <div class="h-full overflow-auto flex flex-col gap-12">
    <div class="flex items-center gap-16 flex-wrap">
      <span class="text-13">
        包名
        <el-input
          v-model="packageName"
          size="small"
          placeholder="默认 basePackage.group.entity"
          style="width: 260px"
        />
      </span>
      <span class="text-13">
        类名
        <el-input
          v-model="className"
          size="small"
          placeholder="默认派生"
          style="width: 140px"
        />
      </span>
      <el-checkbox v-model="useLombok">Lombok @Data</el-checkbox>
      <el-checkbox v-model="includeComment">注释</el-checkbox>
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
