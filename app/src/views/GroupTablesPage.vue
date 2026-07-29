<script setup lang="ts">
// 分组表列表页:点分组节点打开(页签)。表表格 + 工具栏(copy/paste/新增/删除) + 操作列(打开/编辑/复制/删除)。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useClipboardStore } from "@/stores/clipboard";
import type { Table } from "@/types/schema";

const props = defineProps<{ code: string }>();
const router = useRouter();
const store = useProjectStore();
const clipboard = useClipboardStore();

const group = computed(() => store.currentProject?.groups.find((g) => g.code === props.code));
const tables = computed<Table[]>(() => store.currentProject?.tables.filter((t) => t.group === props.code) ?? []);

const selected = ref<Table[]>([]);
const tableRef = ref();
// 表列表变化(如树拖拽移走表) -> 清选中,避免残留引用已不在此分组的表
watch(tables, () => {
  selected.value = [];
  tableRef.value?.clearSelection();
});
function onSelectionChange(rows: Table[]) {
  selected.value = rows;
}

function openTable(row: Table) {
  const t = store.currentProject?.tables.find((x) => x.id === row.id);
  if (t) router.push(store.openTable(t));
}

// copy/paste
function copySelected() {
  if (!selected.value.length) return;
  store.copyTables(selected.value);
  ElMessage.success(`已复制 ${selected.value.length} 张表`);
}
function paste() {
  if (!clipboard.hasTables) return;
  const newCodes = store.pasteTables(props.code);
  ElMessage.success(`已粘贴 ${newCodes.length} 张表`);
}

// 删除表(关页签)
async function deleteTable(tableCode: string) {
  try {
    await ElMessageBox.confirm(`确认删除表 ${tableCode}?`, "删除表", {
      type: "warning", confirmButtonText: "删除", cancelButtonText: "取消",
    });
    const nextPath = store.deleteTable(tableCode);
    if (nextPath) router.push(nextPath);
    ElMessage.success("表已删除");
  } catch { /* 取消 */ }
}
async function deleteSelected() {
  if (!selected.value.length) return;
  try {
    await ElMessageBox.confirm(`确认删除选中的 ${selected.value.length} 张表?`, "删除表", {
      type: "warning", confirmButtonText: "删除", cancelButtonText: "取消",
    });
    for (const t of [...selected.value]) {
      store.deleteTable(t.code);
    }
    selected.value = [];
    ElMessage.success("已删除");
  } catch { /* 取消 */ }
}

// duplicate(同项目一键 _COPY)
function duplicate(tableCode: string) {
  const newCode = store.duplicateTable(tableCode);
  if (!newCode) { ElMessage.error("复制失败"); return; }
  ElMessage.success(`已复制为 ${newCode}`);
}

// ===== 分组编辑/删除 =====
const groupDialogVisible = ref(false);
const groupEditingCode = ref("");
const groupCode = ref("");
const groupName = ref("");
function openEditGroup() {
  if (!group.value) return;
  groupEditingCode.value = group.value.code;
  groupCode.value = group.value.code;
  groupName.value = group.value.name;
  groupDialogVisible.value = true;
}
function confirmGroupDialog() {
  const code = groupCode.value.trim();
  const name = groupName.value.trim();
  if (!code || !name) { ElMessage.warning("code 和名称不能为空"); return; }
  const err = store.updateGroup(groupEditingCode.value, code, name);
  if (err) { ElMessage.error(err); return; }
  ElMessage.success("分组已更新");
  groupDialogVisible.value = false;
  // code 改了 -> 页签 key 变,关旧页签开新
  if (code !== groupEditingCode.value) {
    store.closeTab(`group:${groupEditingCode.value}`);
    router.push(store.openGroup(code));
  }
}
async function deleteGroup() {
  try {
    await ElMessageBox.confirm(`确认删除分组 ${props.code}?`, "删除分组", {
      type: "warning", confirmButtonText: "删除", cancelButtonText: "取消",
    });
    const err = store.deleteGroup(props.code);
    if (err) { ElMessage.error(err); return; }
    const nextPath = store.closeTab(`group:${props.code}`);
    if (nextPath) router.push(nextPath);
    ElMessage.success("分组已删除");
  } catch { /* 取消 */ }
}

// ===== 表新增/编辑 =====
const tableDialogVisible = ref(false);
const tableDialogMode = ref<"add" | "edit">("add");
const tableEditingId = ref("");
const tableCode = ref("");
const tableName = ref("");
const tableComment = ref("");
function openAddTable() {
  tableDialogMode.value = "add";
  tableEditingId.value = "";
  tableCode.value = "";
  tableName.value = "";
  tableComment.value = "";
  tableDialogVisible.value = true;
}
function openEditTable(row: Table) {
  tableDialogMode.value = "edit";
  tableEditingId.value = row.id;
  tableCode.value = row.code;
  tableName.value = row.name;
  tableComment.value = row.comment ?? "";
  tableDialogVisible.value = true;
}
function confirmTableDialog() {
  const code = tableCode.value.trim().toUpperCase();
  const name = tableName.value.trim();
  if (!code || !name) { ElMessage.warning("code 和名称不能为空"); return; }
  if (tableDialogMode.value === "add") {
    const err = store.addTable(code, name, props.code, tableComment.value.trim());
    if (err) { ElMessage.error(err); return; }
    ElMessage.success("表已创建");
    tableDialogVisible.value = false;
    const table = store.currentProject?.tables.find((t) => t.code === code);
    if (table) router.push(store.openTable(table));
  } else {
    const err = store.updateTable(tableEditingId.value, code, name, tableComment.value.trim());
    if (err) { ElMessage.error(err); return; }
    ElMessage.success("表已更新");
    tableDialogVisible.value = false;
  }
}
</script>

<template>
  <div v-if="store.currentProject && group" class="h-full flex flex-col">
    <!-- 工具栏 -->
    <div class="flex items-center px-16 h-48 border-b border-gray-200 flex-shrink-0">
      <span class="font-bold text-14 mr-16">{{ group.name }} ({{ group.code }})</span>
        <el-button v-if="!store.readOnly" size="small" @click="openEditGroup"><span class="i-mdi-pencil w-16 h-16 mr-4" />编辑分组</el-button>
        <el-button v-if="!store.readOnly" size="small" type="danger" @click="deleteGroup"><span class="i-mdi-delete w-16 h-16 mr-4" />删除分组</el-button>
        <el-divider v-if="!store.readOnly" direction="vertical" />
        <el-button v-if="!store.readOnly" size="small" type="primary" @click="openAddTable"><span class="i-mdi-plus w-16 h-16 mr-4" />新增表</el-button>
        <el-button size="small" :disabled="!selected.length" @click="copySelected"><span class="i-mdi-content-copy w-16 h-16 mr-4" />拷贝</el-button>
        <el-button v-if="!store.readOnly" size="small" :disabled="!clipboard.hasTables" @click="paste"><span class="i-mdi-content-paste w-16 h-16 mr-4" />粘贴</el-button>
        <el-button v-if="!store.readOnly" size="small" type="danger" :disabled="!selected.length" @click="deleteSelected"><span class="i-mdi-delete w-16 h-16 mr-4" />删除</el-button>
    </div>

    <!-- 表表格 -->
    <div class="flex-1 min-h-0 p-12">
      <el-table ref="tableRef" :data="tables" border size="small" height="100%" style="width: 100%" @selection-change="onSelectionChange">
        <el-table-column type="selection" width="36" />
        <el-table-column label="编码" min-width="150">
          <template #default="{ row }">
            <span class="text-13 cursor-pointer text-blue-600 hover:underline" @click="openTable(row)">{{ row.code }}</span>
          </template>
        </el-table-column>
        <el-table-column label="名称" min-width="150" prop="name" />
        <el-table-column label="字段数" width="80" align="center">
          <template #default="{ row }">{{ row.fields.length }}</template>
        </el-table-column>
        <el-table-column label="索引数" width="80" align="center">
          <template #default="{ row }">{{ row.indexes?.length ?? 0 }}</template>
        </el-table-column>
        <el-table-column v-if="!store.readOnly" label="操作" width="180" align="center" fixed="right">
          <template #default="{ row }">
            <el-button size="small" link type="primary" @click="openEditTable(row)">编辑</el-button>
            <el-button size="small" link type="primary" @click="duplicate(row.code)">复制</el-button>
            <el-button size="small" link type="danger" @click="deleteTable(row.code)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 分组编辑弹框 -->
    <el-dialog v-model="groupDialogVisible" title="编辑分组" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="code"><el-input v-model="groupCode" /></el-form-item>
        <el-form-item label="名称"><el-input v-model="groupName" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="groupDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmGroupDialog">保存</el-button>
      </template>
    </el-dialog>

    <!-- 表新增/编辑弹框 -->
    <el-dialog v-model="tableDialogVisible" :title="tableDialogMode === 'add' ? '新建表' : '编辑表'" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="code"><el-input v-model="tableCode" /></el-form-item>
        <el-form-item label="名称"><el-input v-model="tableName" /></el-form-item>
        <el-form-item label="备注"><el-input v-model="tableComment" type="textarea" :rows="2" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tableDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmTableDialog">保存</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="分组不存在" class="h-full" />
</template>
