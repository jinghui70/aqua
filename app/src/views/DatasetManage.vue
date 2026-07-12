<script setup lang="ts">
// 数据集管理(§6.4):数据集选择 + 表树 + 数据网格框架。
// 本期 UI 框架;数据行读写待后端 dataset-commands 任务。
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();

// 数据集(本期内存态占位)
const datasets = ref<string[]>(["default"]);
const currentDataset = ref("default");
const hideEmpty = ref(false);

const selectedTable = ref("");

// 表树(按分组)
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
  return p.groups.map((g) => ({
    id: `group:${g.code}`,
    label: g.name,
    type: "group" as const,
    children: p.tables
      .filter((t) => t.group === g.code)
      .map((t) => ({
        id: `table:${t.code}`,
        label: `${t.code} (0)`, // 行数占位
        type: "table" as const,
        tableCode: t.code,
      })),
  }));
});

const currentTable = computed(() =>
  store.currentProject?.tables.find((t) => t.code === selectedTable.value)
);

function onNodeClick(data: TreeNode) {
  if (data.type === "table" && data.tableCode) {
    selectedTable.value = data.tableCode;
  }
}

function notImplemented() {
  ElMessage.info("数据行读写待后端 dataset-commands 任务实现");
}

function addDataset() {
  ElMessage.info("数据集持久化待后端任务");
}
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex flex-col">
    <!-- 顶部:数据集选择 -->
    <div class="flex items-center gap-12 px-16 h-48 border-b border-gray-200 flex-shrink-0">
      <span class="text-13">数据集</span>
      <el-select v-model="currentDataset" size="small" style="width: 160px">
        <el-option v-for="d in datasets" :key="d" :label="d" :value="d" />
      </el-select>
      <el-button size="small" @click="addDataset">新建</el-button>
      <el-button size="small" @click="notImplemented">删除</el-button>
      <el-checkbox v-model="hideEmpty" class="ml-8">隐藏无数据表</el-checkbox>
    </div>

    <!-- 主体:表树 + 数据网格 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 表树 -->
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

      <!-- 数据网格 -->
      <div class="flex-1 overflow-auto p-16">
        <template v-if="currentTable">
          <div class="flex items-center gap-8 mb-12">
            <span class="font-bold text-14">{{ currentTable.code }} 数据</span>
            <div class="flex-1" />
            <el-button size="small" type="primary" @click="notImplemented">新增行</el-button>
            <el-button size="small" @click="notImplemented">从库导入</el-button>
            <el-button size="small" @click="notImplemented">导出表数据</el-button>
            <el-button size="small" type="danger" @click="notImplemented">清空</el-button>
          </div>
          <el-table :data="[]" border size="small" style="width: 100%">
            <el-table-column
              v-for="f in currentTable.fields"
              :key="f.code"
              :prop="f.code"
              :label="f.code"
              min-width="120"
            />
          </el-table>
          <el-empty description="暂无数据(数据行读写待后端实现)" :image-size="60" />
        </template>
        <el-empty v-else description="选择左侧的表" />
      </div>
    </div>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
