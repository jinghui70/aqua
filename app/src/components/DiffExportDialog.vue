<script setup lang="ts">
// diff 导出:选旧版 schema + 方言 + 预览/复制/下载。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { useDatabaseStore } from "@/stores/database";
import { downloadText } from "@/composables/useDownload";
import { pickOpenFile } from "@/composables/useFileDialog";
import type { Project } from "@/types/schema";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

const dialect = ref("mysql");
const oldProjectPath = ref("");
const preview = ref("");

async function pickOldProject() {
  const path = await pickOpenFile();
  if (path) oldProjectPath.value = path;
}

async function doPreview() {
  if (!store.currentProject) return;
  if (!oldProjectPath.value) {
    ElMessage.warning("请选择旧版 schema.json");
    return;
  }
  try {
    const oldProject = await tauri.projectOpen(oldProjectPath.value);
    preview.value = await tauri.generateAlter(oldProject as Project, store.currentProject, dialect.value);
  } catch { /* 已提示 */ }
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  downloadText("alter.sql", preview.value);
}

watch(() => ui.diffExportVisible, (v) => {
  if (v) {
    preview.value = "";
    oldProjectPath.value = "";
  }
});
</script>

<template>
  <el-dialog v-model="ui.diffExportVisible" title="导出 diff (ALTER)" width="70%" :close-on-click-modal="false">
    <div class="flex flex-col gap-12">
      <div class="flex items-center">
        <span class="text-13">方言</span>
        <el-select v-model="dialect" size="small" style="width: 130px; margin-left: 4px">
          <el-option v-for="d in dbStore.generatable" :key="d.name" :label="d.label" :value="d.name" />
        </el-select>
        <el-button size="small" style="margin-left: 16px" @click="pickOldProject">选择旧版</el-button>
        <span class="text-12 text-gray-400" style="margin-left: 8px">{{ oldProjectPath || "未选择" }}</span>
        <div class="flex-1" />
        <el-button size="small" type="primary" @click="doPreview" style="margin-right: 8px">预览</el-button>
        <el-button size="small" @click="copy" :disabled="!preview" style="margin-right: 8px">复制</el-button>
        <el-button size="small" @click="download" :disabled="!preview">下载</el-button>
      </div>
      <el-input v-model="preview" type="textarea" :rows="20" readonly class="font-mono" placeholder="选择旧版后点击预览" />
    </div>
  </el-dialog>
</template>
