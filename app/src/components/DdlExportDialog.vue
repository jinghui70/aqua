<script setup lang="ts">
// DDL 导出:方言 + 选表(对话框树多选)+ 预览/复制/下载。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { useDatabaseStore } from "@/stores/database";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import TableSelectDialog from "./TableSelectDialog.vue";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

const dialect = ref("mysql");
const selectedTables = ref<string[]>([]);
const tableSelectVisible = ref(false);
const preview = ref("");
const dropIfExist = ref(true);

const tables = computed(() => store.currentProject?.tables ?? []);

async function doPreview() {
  if (!store.currentProject) return;
  if (!selectedTables.value.length) {
    ElMessage.warning("请先选择表");
    return;
  }
  try {
    preview.value = await tauri.generateDdl(store.currentProject, dialect.value, { tables: selectedTables.value, dropIfExist: dropIfExist.value });
  } catch { /* 已提示 */ }
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

async function saveFile() {
  const path = await save({
    filters: [{ name: "SQL", extensions: ["sql"] }],
    defaultPath: "ddl.sql",
  });
  if (!path) return;
  try {
    await invoke<void>("write_text_file", { path, content: preview.value });
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

function onTableConfirm(tables: string[]) {
  selectedTables.value = tables;
}

watch(() => ui.ddlExportVisible, (v) => {
  if (v) {
    preview.value = "";
    selectedTables.value = [];
  }
});
// 即时预览(selectedTables 或 dropIfExist 变化)
watch([selectedTables, dropIfExist, dialect], () => {
  if (selectedTables.value.length) doPreview();
  else preview.value = "";
});
</script>

<template>
  <el-dialog v-model="ui.ddlExportVisible" title="导出 DDL" width="70%" :close-on-click-modal="false">
    <div class="flex flex-col gap-12">
      <div class="flex items-center">
        <span class="text-13">方言</span>
        <el-select v-model="dialect" size="small" style="width: 130px; margin-left: 4px">
          <el-option v-for="d in dbStore.generatable" :key="d.name" :label="d.label" :value="d.name" />
        </el-select>
        <el-button size="small" style="margin-left: 16px" @click="tableSelectVisible = true">
          选表{{ selectedTables.length ? ` (${selectedTables.length})` : "" }}
        </el-button>
        <el-checkbox v-model="dropIfExist" style="margin-left: 16px">删表重建</el-checkbox>
        <div class="flex-1" />
        <el-button size="small" @click="copy" :disabled="!preview" style="margin-right: 8px">复制</el-button>
        <el-button size="small" type="primary" @click="saveFile" :disabled="!preview">保存</el-button>
      </div>
      <el-input v-model="preview" type="textarea" :rows="20" readonly class="font-mono" placeholder="选表后即时预览" />
    </div>
    <TableSelectDialog v-model="tableSelectVisible" :selected="selectedTables" @confirm="onTableConfirm" />
  </el-dialog>
</template>
