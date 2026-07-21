<script setup lang="ts">
// 数据集管理(§6.4):打开/新建/保存数据集文件 + 表树(真实行数) + 可编辑数据网格。
// 行数据前端持有(内存),编辑后整体存回;后端 dataset_load/save 无状态。
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { pickDatasetOpen, pickDatasetSave } from "@/composables/useFileDialog";
import { DataType, type DatasetEntry, type Field } from "@/types/schema";

const router = useRouter();
const store = useProjectStore();
const tauri = useTauri();

// 内存态:tableCode(大写) -> 行数组(单元格值统一以字符串显示,保存时按类型规范化)
type Row = Record<string, string>;
const rowsMap = ref<Record<string, Row[]>>({});
const currentPath = ref("");
const hideEmpty = ref(false);
const selectedTable = ref("");

const intTypes = [DataType.Tinyint, DataType.Int, DataType.Long];

function rowCount(tableCode: string): number {
  return rowsMap.value[tableCode.toUpperCase()]?.length ?? 0;
}

function fileName(path: string): string {
  return path ? path.split("/").pop() || path : "(未保存)";
}

// 表树(按分组;可隐藏无数据表)
interface TreeNode {
  id: string;
  label: string;
  type: "group" | "table";
  tableCode?: string;
  children?: TreeNode[];
}
const treeData = computed<TreeNode[]>(() => {
  const p = store.currentProject;
  if (!p) return [];
  return p.groups
    .map((g) => ({
      id: `group:${g.code}`,
      label: g.name,
      type: "group" as const,
      children: p.tables
        .filter((t) => t.group === g.code)
        .filter((t) => !hideEmpty.value || rowCount(t.code) > 0)
        .map((t) => ({
          id: `table:${t.code}`,
          label: `${t.code} (${rowCount(t.code)})`,
          type: "table" as const,
          tableCode: t.code,
        })),
    }))
    .filter((g) => g.children.length > 0 || !hideEmpty.value);
});

const currentTable = computed(() =>
  store.currentProject?.tables.find((t) => t.code === selectedTable.value)
);

// 当前表的行数组(只读展示;增删走 rowsMap 真实数组,避免临时数组丢写)
const currentRows = computed<Row[]>(() => {
  const code = selectedTable.value.toUpperCase();
  return (code && rowsMap.value[code]) || [];
});

function onNodeClick(data: TreeNode) {
  if (data.type === "table" && data.tableCode) selectedTable.value = data.tableCode;
}

// ===== 加载 / 保存 =====
// 后端行值(混合类型) -> 显示态(字符串;null -> "")
function toDisplay(entries: DatasetEntry[]): Record<string, Row[]> {
  const map: Record<string, Row[]> = {};
  for (const e of entries) {
    map[e.table.toUpperCase()] = e.data.map((row) => {
      const r: Row = {};
      for (const [k, v] of Object.entries(row)) r[k.toUpperCase()] = v == null ? "" : String(v);
      return r;
    });
  }
  return map;
}

// 显示态 -> 后端条目(按字段类型规范化:空->null,整数->number,其余->string)
function normalize(field: Field, raw: string): unknown {
  if (raw === "" || raw == null) return null;
  if (intTypes.includes(field.dataType)) {
    const n = Number(raw);
    return Number.isFinite(n) ? n : raw;
  }
  return String(raw);
}
function buildEntries(): DatasetEntry[] {
  const p = store.currentProject!;
  return p.tables.map((t) => {
    const code = t.code.toUpperCase();
    const rows = rowsMap.value[code] ?? [];
    return {
      table: t.code,
      data: rows.map((row) => {
        const r: Record<string, unknown> = {};
        for (const f of t.fields) {
          const key = f.code.toUpperCase();
          r[key] = normalize(f, row[key] ?? "");
        }
        return r;
      }),
    };
  });
}

async function openDataset() {
  if (!store.currentProject) return;
  const path = await pickDatasetOpen();
  if (!path) return;
  try {
    const entries = await tauri.datasetLoad(path, store.currentProject);
    rowsMap.value = toDisplay(entries);
    currentPath.value = path;
    ElMessage.success(`已打开 ${fileName(path)}`);
  } catch {
    /* useTauri 已提示 */
  }
}

async function newDataset() {
  if (!store.currentProject) return;
  const path = await pickDatasetSave();
  if (!path) return;
  rowsMap.value = {};
  try {
    await tauri.datasetSave(path, store.currentProject, buildEntries());
    currentPath.value = path;
    ElMessage.success(`已新建 ${fileName(path)}`);
  } catch {
    /* 已提示 */
  }
}

async function saveDataset() {
  if (!store.currentProject) return;
  let path = currentPath.value;
  if (!path) {
    const picked = await pickDatasetSave();
    if (!picked) return;
    path = picked;
  }
  try {
    await tauri.datasetSave(path, store.currentProject, buildEntries());
    currentPath.value = path;
    ElMessage.success(`已保存到 ${fileName(path)}`);
  } catch {
    /* 已提示 */
  }
}

// ===== 行编辑 =====
function addRow() {
  if (!currentTable.value) return;
  const code = selectedTable.value.toUpperCase();
  if (!rowsMap.value[code]) rowsMap.value[code] = [];
  const row: Row = {};
  for (const f of currentTable.value.fields) row[f.code.toUpperCase()] = "";
  rowsMap.value[code].push(row);
}
function removeRow(idx: number) {
  rowsMap.value[selectedTable.value.toUpperCase()]?.splice(idx, 1);
}
async function clearTable() {
  if (!currentTable.value) return;
  try {
    await ElMessageBox.confirm(
      `确认清空表 ${selectedTable.value} 的全部数据行?`,
      "清空",
      { type: "warning", confirmButtonText: "清空", cancelButtonText: "取消" }
    );
    rowsMap.value[selectedTable.value.toUpperCase()] = [];
  } catch {
    /* 取消 */
  }
}
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex flex-col">
    <!-- 顶部:数据集文件操作 -->
    <div class="flex items-center gap-12 px-16 h-48 border-b border-gray-200 flex-shrink-0">
      <el-button size="small" link @click="router.push('/')">← 返回</el-button>
      <span class="text-13">数据集</span>
      <span class="text-13 text-gray-500 font-mono">{{ fileName(currentPath) }}</span>
      <el-button size="small" @click="newDataset">新建</el-button>
      <el-button size="small" @click="openDataset">打开</el-button>
      <el-button size="small" type="primary" @click="saveDataset">保存</el-button>
      <el-checkbox v-model="hideEmpty" class="ml-8">隐藏无数据表</el-checkbox>
    </div>

    <!-- 主体:表树 + 数据网格 -->
    <div class="flex-1 flex overflow-hidden">
      <div class="w-240 border-r border-gray-200 overflow-y-auto flex-shrink-0 p-4">
        <el-tree
          :data="treeData"
          :props="{ children: 'children', label: 'label' }"
          node-key="id"
          default-expand-all
          :expand-on-click-node="false"
          @node-click="onNodeClick"
        >
          <template #default="{ data }">
            <span class="text-13">
              {{ data.type === "group" ? "📁" : "📄" }} {{ data.label }}
            </span>
          </template>
        </el-tree>
      </div>

      <div class="flex-1 flex flex-col overflow-hidden p-16">
        <template v-if="currentTable">
          <div class="flex items-center gap-8 mb-12 flex-shrink-0">
            <span class="font-bold text-14">{{ currentTable.code }} 数据</span>
            <span class="text-12 text-gray-400">{{ currentRows.length }} 行</span>
            <div class="flex-1" />
            <el-button size="small" type="primary" @click="addRow">新增行</el-button>
            <el-button size="small" type="danger" @click="clearTable">清空</el-button>
          </div>
          <div class="flex-1 min-h-0">
            <el-table :data="currentRows" border size="small" height="100%" style="width: 100%">
              <el-table-column label="#" type="index" width="44" />
              <el-table-column
                v-for="f in currentTable.fields"
                :key="f.code"
                :label="f.code"
                min-width="140"
              >
                <template #default="{ row }">
                  <el-input v-model="row[f.code.toUpperCase()]" size="small" placeholder="null" />
                </template>
              </el-table-column>
              <el-table-column label="操作" width="60" align="center" fixed="right">
                <template #default="{ $index }">
                  <el-button size="small" link type="danger" @click="removeRow($index)">删</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </template>
        <el-empty v-else description="选择左侧的表" />
      </div>
    </div>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
