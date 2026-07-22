<script setup lang="ts">
// 导出弹窗:DDL / diff / StrConst 三合一,预览+复制+下载。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { invoke } from "@tauri-apps/api/core";
import { downloadText } from "@/composables/useDownload";
import { pickOpenFile, pickSaveFile } from "@/composables/useFileDialog";
import type { Project } from "@/types/schema";
import { useDatabaseStore } from "@/stores/database";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

// 通用配置
const dialect = ref("mysql");
const scope = ref<"all" | "group" | "tables">("all");
const selectedGroup = ref("");
const selectedTables = ref<string[]>([]);
const preview = ref("");

// StrConst:无 packageSuffix/className(类名固定 DatabaseConstants,包名按 group)

// diff
const oldProjectPath = ref("");

const title = computed(() => {
  switch (ui.exportKind) {
    case "ddl": return "导出 DDL";
    case "diff": return "导出 diff (ALTER)";
    case "strconst": return "导出字符串变量";
  }
  return "导出";
});

const groups = computed(() => store.currentProject?.groups ?? []);
const tables = computed(() => store.currentProject?.tables ?? []);

function scopeOpts(): { tables?: string[]; group?: string } {
  if (scope.value === "group") return { group: selectedGroup.value };
  if (scope.value === "tables") return { tables: selectedTables.value };
  return {};
}

async function doPreview() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  try {
    if (ui.exportKind === "ddl") {
      preview.value = await tauri.generateDdl(
        store.currentProject,
        dialect.value,
        scopeOpts()
      );
    } else if (ui.exportKind === "strconst") {
      preview.value = await tauri.generateStrConst(store.currentProject, {
        group: scope.value === "group" ? selectedGroup.value : undefined,
      });
    } else if (ui.exportKind === "diff") {
      if (!oldProjectPath.value) {
        ElMessage.warning("请输入旧版 schema.json 路径");
        return;
      }
      const oldProject = await tauri.projectOpen(oldProjectPath.value);
      preview.value = await tauri.generateAlter(
        oldProject as Project,
        store.currentProject,
        dialect.value
      );
    }
  } catch {
    /* 已提示 */
  }
}

async function pickOldProject() {
  const path = await pickOpenFile();
  if (path) oldProjectPath.value = path;
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  const ext = ui.exportKind === "strconst" ? "java" : ui.exportKind === "diff" ? "sql" : "sql";
  const base =
    ui.exportKind === "strconst"
      ? "DatabaseConstants"
      : ui.exportKind === "diff"
        ? "alter"
        : "schema";
  downloadText(`${base}.${ext}`, preview.value);
}

// StrConst 保存:打开文件对话框保存
async function saveStrConst() {
  const path = await pickSaveFile();
  if (!path) return;
  try {
    // 用 Tauri 写文件(downloadText 是 web 下载,桌面用 invoke 写文件)
    await invoke<void>("write_text_file", { path, content: preview.value });
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

// 切换导出类型或打开时清空预览;strconst 即时预览(无预览按钮)
watch(
  () => [ui.exportVisible, ui.exportKind],
  () => {
    if (ui.exportVisible) preview.value = "";
  }
);
// 切到"按分组"时默认选第一个分组
watch(scope, (s) => {
  if (s === "group" && !selectedGroup.value && groups.value.length) {
    selectedGroup.value = groups.value[0].code;
  }
});
watch(
  () => ui.exportKind === "strconst" && ui.exportVisible ? [scope.value, selectedGroup.value] : null,
  () => {
    if (ui.exportKind === "strconst" && ui.exportVisible) doPreview();
  },
  { immediate: true }
);
</script>

<template>
  <el-dialog v-model="ui.exportVisible" :title="title" width="70%" :close-on-click-modal="false">
    <div class="flex flex-col gap-12">
      <!-- 配置区 -->
      <div class="flex items-center gap-16 flex-wrap">
        <!-- 方言(ddl/diff) -->
        <span v-if="ui.exportKind !== 'strconst'" class="flex items-center gap-6 text-13">
          方言
          <el-select v-model="dialect" size="small" style="width: 130px">
            <el-option v-for="d in dbStore.generatable" :key="d.name" :label="d.label" :value="d.name" />
          </el-select>
        </span>

        <!-- diff 对比文件 -->
        <span v-if="ui.exportKind === 'diff'" class="flex items-center gap-6 text-13">
          <el-button size="small" @click="pickOldProject">选择旧版</el-button>
          <span class="text-12 text-gray-400">{{ oldProjectPath || "未选择" }}</span>
        </span>

        <!-- 范围(ddl/strconst) -->
        <template v-if="ui.exportKind !== 'diff'">
          <el-radio-group v-model="scope" size="small">
            <el-radio-button value="all">全部表</el-radio-button>
            <el-radio-button value="group">按分组</el-radio-button>
            <el-radio-button value="tables" v-if="ui.exportKind === 'ddl'">
              指定表
            </el-radio-button>
          </el-radio-group>
          <el-select
            v-if="scope === 'group'"
            v-model="selectedGroup"
            size="small"
            placeholder="选分组"
            style="width: 140px"
          >
            <el-option v-for="g in groups" :key="g.code" :label="g.name" :value="g.code" />
          </el-select>
          <el-select
            v-if="scope === 'tables'"
            v-model="selectedTables"
            multiple
            size="small"
            placeholder="选表"
            style="width: 240px"
          >
            <el-option v-for="t in tables" :key="t.code" :label="t.code" :value="t.code" />
          </el-select>
        </template>
      </div>

      <!-- StrConst:无包名/类名输入(类名固定,包名按 group) -->

      <!-- 操作:strconst 即时预览(无预览按钮)+ 复制/保存;其它 预览/复制/下载 -->
      <div class="flex gap-8">
        <el-button v-if="ui.exportKind !== 'strconst'" size="small" type="primary" @click="doPreview">预览</el-button>
        <el-button size="small" @click="copy" :disabled="!preview">复制</el-button>
        <el-button v-if="ui.exportKind === 'strconst'" size="small" type="primary" @click="saveStrConst" :disabled="!preview">保存</el-button>
        <el-button v-if="ui.exportKind !== 'strconst'" size="small" @click="download" :disabled="!preview">下载</el-button>
      </div>

      <!-- 预览 -->
      <el-input
        v-model="preview"
        type="textarea"
        :rows="18"
        readonly
        class="font-mono"
        placeholder="点击预览生成"
      />
    </div>
  </el-dialog>
</template>
