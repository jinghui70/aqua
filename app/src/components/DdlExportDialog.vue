<script setup lang="ts">
// DDL 导出:方言 + 范围 + 预览/复制/下载。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { useDatabaseStore } from "@/stores/database";
import { downloadText } from "@/composables/useDownload";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

const dialect = ref("mysql");
const scope = ref<"all" | "group" | "tables">("group");
const selectedGroup = ref("");
const selectedTables = ref<string[]>([]);
const preview = ref("");

const groups = computed(() => store.currentProject?.groups ?? []);
const tables = computed(() => store.currentProject?.tables ?? []);

function scopeOpts() {
  if (scope.value === "group") return { group: selectedGroup.value };
  if (scope.value === "tables") return { tables: selectedTables.value };
  return {};
}

async function doPreview() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateDdl(store.currentProject, dialect.value, scopeOpts());
  } catch { /* 已提示 */ }
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  downloadText("schema.sql", preview.value);
}

watch(() => ui.ddlExportVisible, (v) => {
  if (v) {
    preview.value = "";
    scope.value = "group";
    selectedGroup.value = groups.value[0]?.code ?? "";
    selectedTables.value = [];
  }
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
        <el-radio-group v-model="scope" size="small" style="margin-left: 16px">
          <el-radio-button value="all">全部表</el-radio-button>
          <el-radio-button value="group">按分组</el-radio-button>
          <el-radio-button value="tables">指定表</el-radio-button>
        </el-radio-group>
        <el-select v-if="scope === 'group'" v-model="selectedGroup" size="small" placeholder="选分组" style="width: 140px; margin-left: 8px">
          <el-option v-for="g in groups" :key="g.code" :label="g.name" :value="g.code" />
        </el-select>
        <el-select v-if="scope === 'tables'" v-model="selectedTables" multiple size="small" placeholder="选表" style="width: 240px; margin-left: 8px">
          <el-option v-for="t in tables" :key="t.code" :label="t.code" :value="t.code" />
        </el-select>
        <div class="flex-1" />
        <el-button size="small" type="primary" @click="doPreview" style="margin-right: 8px">预览</el-button>
        <el-button size="small" @click="copy" :disabled="!preview" style="margin-right: 8px">复制</el-button>
        <el-button size="small" @click="download" :disabled="!preview">下载</el-button>
      </div>
      <el-input v-model="preview" type="textarea" :rows="20" readonly class="font-mono" placeholder="点击预览生成" />
    </div>
  </el-dialog>
</template>
