<script setup lang="ts">
// 导出弹窗:DDL / diff / StrConst 三合一,预览+复制+下载。
import { computed, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { downloadText } from "@/composables/useDownload";
import type { Project } from "@/types/schema";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();

const dialects = ["mysql", "postgresql", "oracle", "h2"];

// 通用配置
const dialect = ref("mysql");
const scope = ref<"all" | "group" | "tables">("all");
const selectedGroup = ref("");
const selectedTables = ref<string[]>([]);
const preview = ref("");

// StrConst
const packageSuffix = ref("const");
const className = ref("DatabaseConstants");

// diff
const oldProjectPath = ref("");

const title = computed(() => {
  switch (ui.exportKind) {
    case "ddl": return "导出 DDL";
    case "diff": return "导出 diff (ALTER)";
    case "strconst": return "导出 StrConst";
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
        packageSuffix: packageSuffix.value,
        className: className.value,
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
  try {
    const { value } = await ElMessageBox.prompt("旧版 schema.json 路径", "选择对比文件", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      inputPlaceholder: "/path/to/old-schema.json",
    });
    oldProjectPath.value = value;
  } catch {
    /* 取消 */
  }
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

function download() {
  const ext = ui.exportKind === "strconst" ? "java" : ui.exportKind === "diff" ? "sql" : "sql";
  const base =
    ui.exportKind === "strconst"
      ? className.value
      : ui.exportKind === "diff"
        ? "alter"
        : "schema";
  downloadText(`${base}.${ext}`, preview.value);
}

// 切换导出类型或打开时清空预览
watch(
  () => [ui.exportVisible, ui.exportKind],
  () => {
    if (ui.exportVisible) preview.value = "";
  }
);
</script>

<template>
  <el-dialog v-model="ui.exportVisible" :title="title" width="70%">
    <div class="flex flex-col gap-12">
      <!-- 配置区 -->
      <div class="flex items-center gap-16 flex-wrap">
        <!-- 方言(ddl/diff) -->
        <span v-if="ui.exportKind !== 'strconst'" class="flex items-center gap-6 text-13">
          方言
          <el-select v-model="dialect" size="small" style="width: 130px">
            <el-option v-for="d in dialects" :key="d" :label="d" :value="d" />
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

      <!-- StrConst 包名/类名 -->
      <div v-if="ui.exportKind === 'strconst'" class="flex items-center gap-16 text-13">
        <span class="flex items-center gap-6">
          包名后缀
          <el-input v-model="packageSuffix" size="small" style="width: 140px" />
        </span>
        <span class="flex items-center gap-6">
          类名
          <el-input v-model="className" size="small" style="width: 180px" />
        </span>
      </div>

      <!-- 操作 -->
      <div class="flex gap-8">
        <el-button size="small" type="primary" @click="doPreview">预览</el-button>
        <el-button size="small" @click="copy" :disabled="!preview">复制</el-button>
        <el-button size="small" @click="download" :disabled="!preview">下载</el-button>
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
