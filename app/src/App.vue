<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { useProject } from "@/composables/useProject";
import { useTauri } from "@/composables/useTauri";
import { DataType } from "@/types/schema";

const project = useProject();
const tauri = useTauri();

const genDialogVisible = ref(false);
const genOutput = ref("");
const genType = ref<"ddl" | "java">("ddl");
const ddlDialect = ref("mysql");
const javaTable = ref("");

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

async function handleGenerate(type: "ddl" | "java") {
  if (!project.currentProject.value) {
    ElMessage.warning("请先打开项目");
    return;
  }
  genType.value = type;
  try {
    if (type === "ddl") {
      genOutput.value = await tauri.generateDdl(
        project.currentProject.value,
        ddlDialect.value
      );
    } else {
      const table = javaTable.value || project.selectedTableCode.value;
      if (!table) {
        ElMessage.warning("请先选择表");
        return;
      }
      genOutput.value = await tauri.generateJava(
        project.currentProject.value,
        table
      );
    }
    genDialogVisible.value = true;
  } catch {
    /* 错误已处理 */
  }
}

async function copyOutput() {
  await navigator.clipboard.writeText(genOutput.value);
  ElMessage.success("已复制到剪贴板");
}

// === 导入向导 ===
const importDialogVisible = ref(false);
const importing = ref(false);
const dbConfig = ref({
  dialect: "mysql",
  host: "localhost",
  port: 3306,
  user: "root",
  password: "",
  database: "",
  schema: "",
});
const importBasePackage = ref("com.example");

async function handleTestConnection() {
  try {
    const msg = await tauri.testConnection({
      ...dbConfig.value,
      schema: dbConfig.value.schema || undefined,
    } as never);
    ElMessage.success(msg);
  } catch {
    /* 错误已处理 */
  }
}

async function handleImport() {
  if (!dbConfig.value.database) {
    ElMessage.warning("请填写数据库名");
    return;
  }
  importing.value = true;
  try {
    const imported = await tauri.importFromDb(
      { ...dbConfig.value, schema: dbConfig.value.schema || undefined } as never,
      importBasePackage.value
    );
    project.currentProject.value = imported;
    project.currentPath.value = "";
    project.selectTable(imported.tables[0]?.code ?? "");
    importDialogVisible.value = false;
    ElMessage.success(`导入成功: ${imported.tables.length} 个表`);
  } catch {
    /* 错误已处理 */
  } finally {
    importing.value = false;
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

function handleDeleteField(index: number) {
  if (!project.currentTable.value) return;
  project.currentTable.value.fields.splice(index, 1);
}

function handleDeleteTable() {
  if (!project.currentProject.value || !project.currentTable.value) return;
  if (!confirm(`确认删除表 ${project.currentTable.value.code}?`)) return;
  const idx = project.currentProject.value.tables.findIndex(
    (t) => t.code === project.currentTable.value?.code
  );
  if (idx >= 0) {
    project.currentProject.value.tables.splice(idx, 1);
    project.selectTable("");
  }
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
        <el-button type="success" @click="handleGenerate('ddl')">生成 DDL</el-button>
        <el-select
          v-model="javaTable"
          placeholder="选择表"
          style="width: 180px"
        >
          <el-option
            v-for="t in project.currentProject.value?.tables ?? []"
            :key="t.code"
            :label="t.code"
            :value="t.code"
          />
        </el-select>
        <el-button type="warning" @click="handleGenerate('java')">生成 Java</el-button>
        <el-divider direction="vertical" />
        <el-button type="info" @click="importDialogVisible = true">
          从数据库导入
        </el-button>
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
              <el-input
                v-model="project.currentTable.value.code"
                style="width: 200px"
                placeholder="表名"
              />
              <el-input
                v-model="project.currentTable.value.name"
                style="width: 140px"
                placeholder="中文名"
              />
              <el-input
                v-model="project.currentTable.value.group"
                style="width: 120px"
                placeholder="分组"
              />
              <el-button size="small" type="primary" @click="handleAddField">
                + 新增字段
              </el-button>
              <el-button size="small" type="danger" @click="handleDeleteTable">
                删除表
              </el-button>
            </div>
            <el-table
              :data="project.currentTable.value.fields"
              border
              style="width: 100%"
            >
              <el-table-column prop="code" label="字段名" width="160">
                <template #default="{ row }">
                  <el-input v-model="row.code" size="small" />
                </template>
              </el-table-column>
              <el-table-column prop="name" label="中文名" width="120">
                <template #default="{ row }">
                  <el-input v-model="row.name" size="small" />
                </template>
              </el-table-column>
              <el-table-column label="类型" width="180">
                <template #default="{ row }">
                  <el-select v-model="row.dataType" size="small" style="width: 110px">
                    <el-option
                      v-for="dt in dataTypes"
                      :key="dt"
                      :label="dt"
                      :value="dt"
                    />
                  </el-select>
                  <el-input-number
                    v-if="row.dataType === 'VARCHAR'"
                    v-model="row.length"
                    size="small"
                    :min="1"
                    style="width: 60px"
                  />
                  <span v-if="row.dataType === 'DECIMAL'">
                    <el-input-number
                      v-model="row.precision"
                      size="small"
                      :min="1"
                      style="width: 55px"
                    />,
                    <el-input-number
                      v-model="row.scale"
                      size="small"
                      :min="0"
                      style="width: 55px"
                    />
                  </span>
                </template>
              </el-table-column>
              <el-table-column label="主键" width="60">
                <template #default="{ row }">
                  <el-checkbox v-model="row.isKey" />
                </template>
              </el-table-column>
              <el-table-column label="非空" width="60">
                <template #default="{ row }">
                  <el-checkbox v-model="row.notNull" />
                </template>
              </el-table-column>
              <el-table-column label="操作" width="80">
                <template #default="{ $index }">
                  <el-button
                    size="small"
                    type="danger"
                    link
                    @click="handleDeleteField($index)"
                  >
                    删除
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </template>
          <el-empty v-else description="请选择左侧的表" />
        </el-main>
      </el-container>
    </el-main>

    <!-- 生成结果预览对话框 -->
    <el-dialog
      v-model="genDialogVisible"
      :title="genType === 'ddl' ? 'DDL 预览' : 'Java 实体预览'"
      width="70%"
    >
      <div style="margin-bottom: 12px">
        <el-button size="small" @click="copyOutput">复制</el-button>
      </div>
      <el-input v-model="genOutput" type="textarea" :rows="20" readonly />
    </el-dialog>

    <!-- 导入向导对话框 -->
    <el-dialog v-model="importDialogVisible" title="从数据库导入" width="500px">
      <el-form :model="dbConfig" label-width="90px">
        <el-form-item label="数据库类型">
          <el-select v-model="dbConfig.dialect" style="width: 100%">
            <el-option label="MySQL" value="mysql" />
            <el-option label="PostgreSQL" value="postgresql" />
            <el-option label="Oracle" value="oracle" />
            <el-option label="H2" value="h2" />
          </el-select>
        </el-form-item>
        <el-form-item label="主机">
          <el-input v-model="dbConfig.host" placeholder="localhost" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="dbConfig.port" :min="1" :max="65535" />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="dbConfig.user" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input v-model="dbConfig.password" type="password" show-password />
        </el-form-item>
        <el-form-item label="数据库">
          <el-input v-model="dbConfig.database" placeholder="数据库名/schema" />
        </el-form-item>
        <el-form-item label="基础包名">
          <el-input v-model="importBasePackage" placeholder="com.example" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="handleTestConnection">测试连接</el-button>
        <el-button type="primary" :loading="importing" @click="handleImport">
          导入
        </el-button>
        <el-button @click="importDialogVisible = false">取消</el-button>
      </template>
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
