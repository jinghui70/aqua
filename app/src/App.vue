<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { useProject } from "@/composables/useProject";
import { useTauri } from "@/composables/useTauri";
import { DataType } from "@/types/schema";

const project = useProject();
const tauri = useTauri();

const ddlDialogVisible = ref(false);
const ddlOutput = ref("");
const ddlDialect = ref("mysql");

const dialects = [
  { label: "MySQL", value: "mysql" },
  { label: "PostgreSQL", value: "postgresql" },
  { label: "Oracle", value: "oracle" },
  { label: "H2", value: "h2" },
];

const dataTypes: DataType[] = [
  DataType.Varchar,
  DataType.Clob,
  DataType.Tinyint,
  DataType.Int,
  DataType.Long,
  DataType.Decimal,
  DataType.Date,
  DataType.Datetime,
  DataType.Blob,
];

async function handleOpen() {
  const path = prompt("请输入 schema.json 路径");
  if (!path) return;
  try {
    await project.openProject(path);
    ElMessage.success(`已打开 ${path}`);
  } catch {
    /* 错误已在 useTauri 处理 */
  }
}

async function handleSave() {
  if (!project.currentProject.value) {
    ElMessage.warning("请先打开或新建项目");
    return;
  }
  const path = project.currentPath.value || prompt("请输入保存路径");
  if (!path) return;
  try {
    await project.saveProject(path);
    ElMessage.success(`已保存到 ${path}`);
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

function handleNew() {
  project.newProject();
  ElMessage.success("已新建项目");
}

async function handleGenerateDdl() {
  if (!project.currentProject.value) {
    ElMessage.warning("请先打开项目");
    return;
  }
  try {
    ddlOutput.value = await tauri.generateDdl(
      project.currentProject.value,
      ddlDialect.value
    );
    ddlDialogVisible.value = true;
  } catch {
    /* 错误已处理 */
  }
}

function handleAddTable() {
  if (!project.currentProject.value) return;
  const code = prompt("表名(大写蛇形,如 SYS_USER)");
  if (!code) return;
  project.currentProject.value.tables.push({
    code: code.toUpperCase(),
    name: code,
    group: "default",
    fields: [],
  });
  project.selectTable(code.toUpperCase());
}

function handleAddField() {
  if (!project.currentTable.value) return;
  project.currentTable.value.fields.push({
    prop: "newField",
    code: "NEW_FIELD",
    name: "新字段",
    dataType: DataType.Varchar,
    length: 64,
  });
}
</script>

<template>
  <div class="app-container">
    <!-- 顶部工具栏 -->
    <el-header class="toolbar">
      <div class="toolbar-left">
        <h1 class="title">aqua v2</h1>
        <span v-if="project.currentPath.value" class="path">
          {{ project.currentPath.value }}
        </span>
      </div>
      <div class="toolbar-right">
        <el-button @click="handleNew">新建</el-button>
        <el-button @click="handleOpen">打开</el-button>
        <el-button type="primary" @click="handleSave">保存</el-button>
        <el-divider direction="vertical" />
        <el-select v-model="ddlDialect" style="width: 140px">
          <el-option
            v-for="d in dialects"
            :key="d.value"
            :label="d.label"
            :value="d.value"
          />
        </el-select>
        <el-button type="success" @click="handleGenerateDdl">生成 DDL</el-button>
      </div>
    </el-header>

    <!-- 主体:左侧表树 + 右侧字段面板 -->
    <el-main class="main-body">
      <el-container style="height: 100%">
        <!-- 左侧表树 -->
        <el-aside width="260px" class="table-aside">
          <div class="aside-header">
            <span>表列表</span>
            <el-button size="small" type="primary" link @click="handleAddTable">
              + 新增表
            </el-button>
          </div>
          <el-menu
            :default-active="project.selectedTableCode.value"
            @select="project.selectTable"
          >
            <el-menu-item
              v-for="t in project.currentProject.value?.tables ?? []"
              :key="t.code"
              :index="t.code"
            >
              <span>{{ t.code }}</span>
              <span class="table-name">{{ t.name }}</span>
            </el-menu-item>
          </el-menu>
          <el-empty
            v-if="!project.currentProject.value?.tables.length"
            description="暂无表,请新建或导入"
          />
        </el-aside>

        <!-- 右侧字段面板 -->
        <el-main class="field-panel">
          <template v-if="project.currentTable.value">
            <div class="panel-header">
              <h2>{{ project.currentTable.value.code }}</h2>
              <span class="table-meta">
                {{ project.currentTable.value.name }} · 分组:
                {{ project.currentTable.value.group }}
              </span>
              <el-button size="small" type="primary" @click="handleAddField">
                + 新增字段
              </el-button>
            </div>
            <el-table
              :data="project.currentTable.value.fields"
              border
              style="width: 100%"
            >
              <el-table-column prop="code" label="字段名" width="160" />
              <el-table-column prop="name" label="中文名" width="120" />
              <el-table-column label="类型" width="160">
                <template #default="{ row }">
                  <span>{{ row.dataType }}</span>
                  <span v-if="row.length">({{ row.length }})</span>
                  <span v-if="row.precision"
                    >({{ row.precision }}, {{ row.scale ?? 0 }})</span
                  >
                </template>
              </el-table-column>
              <el-table-column label="主键" width="60">
                <template #default="{ row }">
                  <el-tag v-if="row.isKey" type="danger" size="small">PK</el-tag>
                </template>
              </el-table-column>
              <el-table-column label="非空" width="60">
                <template #default="{ row }">
                  <el-tag v-if="row.notNull" size="small">NN</el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="comment" label="备注" />
            </el-table>
          </template>
          <el-empty v-else description="请选择左侧的表" />
        </el-main>
      </el-container>
    </el-main>

    <!-- DDL 预览对话框 -->
    <el-dialog v-model="ddlDialogVisible" title="DDL 预览" width="70%">
      <el-input
        v-model="ddlOutput"
        type="textarea"
        :rows="20"
        readonly
      />
    </el-dialog>
  </div>
</template>

<style scoped>
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e4e7ed;
  padding: 0 20px;
}
.toolbar-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}
.title {
  font-size: 18px;
  margin: 0;
}
.path {
  color: #909399;
  font-size: 13px;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.main-body {
  flex: 1;
  padding: 0;
  overflow: hidden;
}
.table-aside {
  border-right: 1px solid #e4e7ed;
  display: flex;
  flex-direction: column;
}
.aside-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid #e4e7ed;
  font-weight: bold;
}
.aside-header + .el-menu {
  flex: 1;
  overflow-y: auto;
}
.table-name {
  color: #909399;
  font-size: 12px;
  margin-left: 8px;
}
.field-panel {
  padding: 20px;
  overflow-y: auto;
}
.panel-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 16px;
}
.panel-header h2 {
  margin: 0;
}
.table-meta {
  color: #909399;
  font-size: 13px;
  flex: 1;
}
</style>
