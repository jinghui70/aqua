<script setup lang="ts">
// 数据集管理:目录扫描下拉 + DBeaver 式编辑(dirty 保存/取消)+ 表头中文名。
// .data JSONL 文件,不存表结构(用主项目)。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { nextTick } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { type DatasetEntry, type Field } from "@/types/schema";

const router = useRouter();
const store = useProjectStore();
const tauri = useTauri();

type Row = Record<string, string>;
const intTypes = ["TINYINT", "INT", "LONG"];

// 数据集列表 + 选中
const datasets = ref<{ name: string; path: string }[]>([]);
const selectedPath = ref("");
const hideEmpty = ref(false);
const selectedTable = ref("");

// 行数据
const rowsMap = ref<Record<string, Row[]>>({});
const dirty = ref(false);
let suppressDirty = false;
const originalRowsMap = ref<Record<string, Row[]>>({});

watch(rowsMap, () => { if (!suppressDirty) dirty.value = true; }, { deep: true });

// 新建弹窗
const newVisible = ref(false);
const newName = ref("");

// ===== 数据集列表 =====
async function loadDatasets() {
  if (!store.currentPath) return;
  try {
    datasets.value = await tauri.scanDatasets(store.currentPath);
  } catch { /* 已提示 */ }
}

async function onDatasetChange() {
  if (!selectedPath.value || !store.currentProject) return;
  try {
    const entries = await tauri.datasetLoad(selectedPath.value, store.currentProject);
    suppressDirty = true;
    rowsMap.value = toDisplay(entries);
    originalRowsMap.value = JSON.parse(JSON.stringify(rowsMap.value));
    dirty.value = false;
    await nextTick();
    suppressDirty = false;
  } catch { /* 已提示 */ }
}

async function confirmNew() {
  const name = newName.value.trim();
  if (!name) { ElMessage.warning("请输入数据集名"); return; }
  if (datasets.value.some((d) => d.name === name)) { ElMessage.error(`${name} 已存在`); return; }
  try {
    await tauri.createDataset(store.currentPath, name);
    newVisible.value = false;
    newName.value = "";
    await loadDatasets();
    // 选中新建的
    const created = datasets.value.find((d) => d.name === name);
    if (created) {
      selectedPath.value = created.path;
      await onDatasetChange();
    }
    ElMessage.success("已创建");
  } catch { /* 已提示 */ }
}

// ===== 保存 / 取消 =====
async function saveDataset() {
  if (!store.currentProject || !selectedPath.value) return;
  try {
    await tauri.datasetSave(selectedPath.value, store.currentProject, buildEntries());
    originalRowsMap.value = JSON.parse(JSON.stringify(rowsMap.value));
    dirty.value = false;
    ElMessage.success("已保存");
  } catch { /* 已提示 */ }
}

function cancelEdit() {
  suppressDirty = true;
  rowsMap.value = JSON.parse(JSON.stringify(originalRowsMap.value));
  dirty.value = false;
  nextTick(() => { suppressDirty = false; });
}

// ===== 行编辑 =====
function rowCount(tableCode: string): number {
  return rowsMap.value[tableCode.toUpperCase()]?.length ?? 0;
}

const treeData = computed(() => {
  const p = store.currentProject;
  if (!p) return [];
  return p.groups.map((g) => ({
    id: `group:${g.code}`,
    label: g.name,
    type: "group" as const,
    children: p.tables
      .filter((t) => t.group === g.code)
      .filter((t) => !hideEmpty.value || rowCount(t.code) > 0)
      .map((t) => ({
        id: `table:${t.code}`,
        label: `${t.name} (${rowCount(t.code)})`,
        type: "table" as const,
        tableCode: t.code,
      })),
  })).filter((g) => g.children.length > 0 || !hideEmpty.value);
});

const currentTable = computed(() =>
  store.currentProject?.tables.find((t) => t.code === selectedTable.value)
);

const currentRows = computed<Row[]>(() => {
  const code = selectedTable.value.toUpperCase();
  return (code && rowsMap.value[code]) || [];
});

function onNodeClick(data: { type: string; tableCode?: string }) {
  if (data.type === "table" && data.tableCode) selectedTable.value = data.tableCode;
}

// ===== 类型转换(保留原逻辑)=====
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
    await ElMessageBox.confirm(`确认清空表 ${selectedTable.value} 的全部数据行?`, "清空", {
      type: "warning", confirmButtonText: "清空", cancelButtonText: "取消",
    });
    rowsMap.value[selectedTable.value.toUpperCase()] = [];
  } catch { /* 取消 */ }
}

// 进入页面扫描数据集
loadDatasets();
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex flex-col">
    <!-- 顶部:数据集下拉 + 新建 + dirty 保存/取消 -->
    <div class="flex items-center px-16 h-48 border-b border-gray-200 flex-shrink-0">
      <el-button size="small" link @click="router.push('/')">← 返回</el-button>
      <span class="text-13" style="margin-left: 8px">数据集</span>
      <el-select
        v-model="selectedPath"
        size="small"
        placeholder="选择数据集"
        style="width: 200px; margin-left: 8px"
        @change="onDatasetChange"
      >
        <el-option v-for="d in datasets" :key="d.path" :label="d.name" :value="d.path" />
      </el-select>
      <el-button size="small" style="margin-left: 8px" @click="newVisible = true">新建</el-button>
      <template v-if="dirty">
        <el-button size="small" type="primary" style="margin-left: 8px" @click="saveDataset">保存</el-button>
        <el-button size="small" style="margin-left: 8px" @click="cancelEdit">取消</el-button>
      </template>
      <div class="flex-1" />
      <el-checkbox v-model="hideEmpty">隐藏无数据表</el-checkbox>
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
            <span class="text-13">{{ data.type === "group" ? "📁" : "📄" }} {{ data.label }}</span>
          </template>
        </el-tree>
      </div>

      <div class="flex-1 flex flex-col overflow-hidden p-16">
        <template v-if="currentTable">
          <div class="flex items-center gap-8 mb-12 flex-shrink-0">
            <span class="font-bold text-14">{{ currentTable.name }}</span>
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
                :label="f.name"
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

    <!-- 新建弹窗 -->
    <el-dialog v-model="newVisible" title="新建数据集" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="数据集名">
          <el-input v-model="newName" placeholder="如 dev / test" />
        </el-form-item>
      </el-form>
      <div class="text-12 text-gray-400 mb-8">
        数据集文件 = 主文件名.数据集名.data(JSONL 格式,Git 友好)<br>
        用于初始数据和测试数据,不存表结构(结构用主项目)
      </div>
      <template #footer>
        <el-button @click="newVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmNew">创建</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
