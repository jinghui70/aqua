<script setup lang="ts">
// DDL Tab: 选方言 + 单表 DDL 实时预览 + 复制/下载。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useTauri } from "@/composables/useTauri";
import { useProjectStore } from "@/stores/project";
import { useDatabaseStore } from "@/stores/database";
import { downloadText } from "@/composables/useDownload";

const props = defineProps<{ tableCode: string; active: boolean }>();

const tauri = useTauri();
const store = useProjectStore();
const dbStore = useDatabaseStore();

const dialect = ref("mysql");
const preview = ref("");

async function refresh() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateDdl(store.currentProject, dialect.value, {
      tables: [props.tableCode],
    });
  } catch {
    /* 已提示 */
  }
}

// 方言 / 表变化实时刷新
watch([dialect, () => props.tableCode], refresh, { immediate: true });
// 切回本 tab 时重新生成,同步字段/索引的改动
watch(() => props.active, (a) => a && refresh());

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  downloadText(`${props.tableCode}.${dialect.value}.sql`, preview.value);
}
</script>

<template>
  <div class="h-full flex flex-col gap-12">
    <div class="flex items-center gap-16 flex-wrap flex-shrink-0">
      <span class="text-13">
        方言
        <el-select v-model="dialect" size="small" style="width: 130px">
          <el-option v-for="d in dbStore.generatable" :key="d.name" :label="d.label" :value="d.name" />
        </el-select>
      </span>
      <div class="flex-1" />
      <el-button size="small" @click="copy">复制</el-button>
      <el-button size="small" type="primary" @click="download">下载</el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-input
        v-model="preview"
        type="textarea"
        resize="none"
        readonly
        class="font-mono h-full"
        :input-style="{ height: '100%' }"
      />
    </div>
  </div>
</template>
