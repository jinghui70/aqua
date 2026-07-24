<script setup lang="ts">
// 数据集管理:目录扫描下拉 + DBeaver 式编辑(dirty 保存/取消)+ 表头中文名。
// .data JSONL 文件,不存表结构(用主项目)。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { nextTick } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useDataSourceStore } from "@/stores/datasource";
import { useTauri } from "@/composables/useTauri";
import { type DatasetEntry, type Field, type DbConfig } from "@/types/schema";
import TableSelectDialog from "@/components/TableSelectDialog.vue";

const router = useRouter();
const store = useProjectStore();
const dsStore = useDataSourceStore();
const tauri = useTauri();

type Row = Record<string, string>;
const intTypes = ["TINYINT", "INT", "LONG"];

// 数据集列表 + 选中
const datasets = ref<{ name: string; path: string }[]>([]);
const selectedPath = ref("");
const lastLoadedPath = ref(""); // 打开取消时回退到此
const migratedDirty = ref(false); // 打开时有结构迁移 -> 取消编辑也保持 dirty
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

// 导入导出:选数据源 + 选表
const ioVisible = ref(false);
const ioMode = ref<"import" | "export">("import");
const ioSource = ref("");
const ioSources = computed(() => dsStore.sources);
const ioTables = ref<string[]>([]);
const ioTableSelectVisible = ref(false);
// 导出选表时只显示有数据的表(基于当前数据集行数)
const ioDataRows = computed(() => {
  const m: Record<string, number> = {};
  for (const [code, rows] of Object.entries(rowsMap.value)) {
    m[code] = rows.length;
  }
  return m;
});

async function openImport() {
  if (!selectedPath.value) { ElMessage.warning("请先选择数据集"); return; }
  ioMode.value = "import";
  ioSource.value = ioSources.value[0]?.sourceName ?? "";
  ioTables.value = [];
  ioVisible.value = true;
}

async function openExport() {
  if (!selectedPath.value) { ElMessage.warning("请先选择数据集"); return; }
  ioMode.value = "export";
  ioSource.value = ioSources.value[0]?.sourceName ?? "";
  ioTables.value = [];
  ioVisible.value = true;
}

function onIoTableConfirm(tables: string[]) {
  ioTables.value = tables;
}

async function confirmIO() {
  if (!ioSource.value) { ElMessage.warning("请选择数据源"); return; }
  if (!store.currentProject || !selectedPath.value) return;
  const source = ioSources.value.find((s) => s.sourceName === ioSource.value);
  if (!source) return;
  const { sourceName: _, ...config } = source;
  const tables = ioTables.value.length ? ioTables.value : undefined;
  try {
    if (ioMode.value === "import") {
      const result = await tauri.datasetImport(selectedPath.value, store.currentProject, config, tables);
      ElMessage.success(`导入完成: ${result.total} 行`);
      await onDatasetChange(); // 重新加载
    } else {
      try {
        await ElMessageBox.confirm("导出将 TRUNCATE + INSERT 覆盖数据库表数据,确认?", "警告", {
          type: "warning", confirmButtonText: "覆盖导出", cancelButtonText: "取消",
        });
      } catch { return; }
      const result = await tauri.datasetExport(selectedPath.value, store.currentProject, config, true, tables);
      ElMessage.success(`导出完成: ${result.affected} 行`);
    }
    ioVisible.value = false;
  } catch { /* 已提示 */ }
}

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
    const { entries, diffs } = await tauri.datasetLoad(selectedPath.value, store.currentProject);
    if (diffs.length) {
      const msg = diffs.map((d) => {
        const parts: string[] = [];
        if (d.removed.length) parts.push(`删除 [${d.removed.join(", ")}]`);
        if (d.added.length) parts.push(`新增 [${d.added.join(", ")}]`);
        return `表 ${d.table}：${parts.join("，")}`;
      }).join("\n");
      try {
        await ElMessageBox.confirm(
          `检测到数据集结构与当前项目不一致：\n${msg}\n\n按新结构打开?(删除的字段将丢弃,新增的字段填空值)`,
          "结构变化",
          { type: "warning", confirmButtonText: "按新结构打开", cancelButtonText: "取消" },
        );
      } catch {
        selectedPath.value = lastLoadedPath.value; // 取消,回退到上次打开的
        return;
      }
    }
    suppressDirty = true;
    rowsMap.value = toDisplay(entries);
    originalRowsMap.value = JSON.parse(JSON.stringify(rowsMap.value));
    dirty.value = (migratedDirty.value = diffs.length > 0); // 有迁移 -> 需保存
    lastLoadedPath.value = selectedPath.value;
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
    migratedDirty.value = false; // 保存后文件已是新结构
    dirty.value = false;
    ElMessage.success("已保存");
  } catch { /* 已提示 */ }
}

function cancelEdit() {
  suppressDirty = true;
  rowsMap.value = JSON.parse(JSON.stringify(originalRowsMap.value));
  dirty.value = migratedDirty.value; // 有迁移时取消仍需保存
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
      <el-button size="small" style="margin-left: 8px" @click="openImport" :disabled="!selectedPath">导入</el-button>
      <el-button size="small" style="margin-left: 8px" @click="openExport" :disabled="!selectedPath">导出</el-button>
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

    <!-- 导入/导出弹窗 -->
    <el-dialog
      v-model="ioVisible"
      :title="ioMode === 'import' ? '从数据库导入' : '导出到数据库'"
      width="420px"
      :close-on-click-modal="false"
    >
      <el-form label-width="80px">
        <el-form-item label="数据源">
          <el-select v-model="ioSource" placeholder="选数据源" style="width: 100%">
            <el-option v-for="s in ioSources" :key="s.sourceName" :label="s.sourceName" :value="s.sourceName" />
          </el-select>
        </el-form-item>
        <el-form-item label="表">
          <el-button size="small" @click="ioTableSelectVisible = true">
            选表{{ ioTables.length ? ` (${ioTables.length})` : "" }}
          </el-button>
          <span v-if="!ioTables.length" class="text-12 text-gray-400 ml-8">未选则全部表</span>
        </el-form-item>
      </el-form>
      <div v-if="ioMode === 'export'" class="text-12 text-red-400 mb-8">
        ⚠️ 导出将 TRUNCATE + INSERT 覆盖数据库表数据
      </div>
      <div v-else class="text-12 text-gray-400 mb-8">
        导入将覆盖数据集当前数据
      </div>
      <template #footer>
        <el-button @click="ioVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmIO">{{ ioMode === "import" ? "导入" : "导出" }}</el-button>
      </template>
    </el-dialog>
    <TableSelectDialog
      v-model="ioTableSelectVisible"
      :selected="ioTables"
      :data-rows="ioMode === 'export' ? ioDataRows : undefined"
      @confirm="onIoTableConfirm"
    />
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
