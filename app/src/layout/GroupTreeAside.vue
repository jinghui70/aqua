<script setup lang="ts">
// 左侧分组树:分组 > 表 两层。点分组开表列表页,点表开编辑标签,拖拽排序/移动,新建分组。
// 表/分组编辑/删除/复制移到 GroupTablesPage(点分组打开)。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
const router = useRouter();

const filterText = ref("");
const treeRef = ref();

interface TreeNode {
  id: string;
  label: string;
  type: "group" | "table";
  groupCode: string;
  tableCode?: string;
  children?: TreeNode[];
}

const treeData = computed<TreeNode[]>(() => {
  const p = store.currentProject;
  if (!p) return [];
  return p.groups.map((g) => ({
    id: `group:${g.code}`,
    label: `${g.name} (${g.code})`,
    type: "group" as const,
    groupCode: g.code,
    children: p.tables
      .filter((t) => t.group === g.code)
      .map((t) => ({
        id: `table:${t.id}`,
        label: `${t.code} ${t.name}`,
        type: "table" as const,
        groupCode: g.code,
        tableCode: t.code,
      })),
  }));
});

const treeProps = { children: "children", label: "label" };

function filterNode(value: string, data: TreeNode) {
  if (!value) return true;
  return data.label.toLowerCase().includes(value.toLowerCase());
}

watch(filterText, (v) => treeRef.value?.filter(v));

// 点击节点:分组 -> 开表列表页;表 -> 开编辑标签
function onNodeClick(data: TreeNode) {
  if (data.type === "group") {
    router.push(store.openGroup(data.groupCode));
  } else if (data.type === "table" && data.tableCode) {
    const table = store.currentProject?.tables.find((t) => t.code === data.tableCode);
    if (table) router.push(store.openTable(table));
  }
}

// ===== 拖拽:层级约束 + 落位写回 store =====
/* eslint-disable @typescript-eslint/no-explicit-any */
function allowDrop(draggingNode: any, dropNode: any, type: string): boolean {
  const drag = draggingNode.data as TreeNode;
  const drop = dropNode.data as TreeNode;
  if (drag.type === "group") {
    return type !== "inner" && drop.type === "group";
  }
  if (type === "inner") return drop.type === "group";
  return drop.type === "table";
}

function onNodeDrop(draggingNode: any, dropNode: any, dropType: string) {
  const drag = draggingNode.data as TreeNode;
  const drop = dropNode.data as TreeNode;
  if (drag.type === "group") {
    store.reorderGroups(drag.groupCode, drop.groupCode, dropType as "before" | "after");
    return;
  }
  if (!drag.tableCode) return;
  if (dropType === "inner") {
    store.moveTable(drag.tableCode, drop.groupCode);
  } else {
    if (!drop.tableCode) return;
    store.moveTable(drag.tableCode, drop.groupCode, {
      refCode: drop.tableCode,
      type: dropType as "before" | "after",
    });
  }
}
/* eslint-enable @typescript-eslint/no-explicit-any */

// ===== 新建分组 =====
const groupDialogVisible = ref(false);
const groupCode = ref("");
const groupName = ref("");

function openAddGroup() {
  groupCode.value = "";
  groupName.value = "";
  groupDialogVisible.value = true;
}
function confirmGroupDialog() {
  const code = groupCode.value.trim();
  const name = groupName.value.trim();
  if (!code || !name) {
    ElMessage.warning("code 和名称不能为空");
    return;
  }
  const err = store.addGroup(code, name);
  if (err) {
    ElMessage.error(err);
    return;
  }
  ElMessage.success("分组已创建");
  groupDialogVisible.value = false;
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 头部:标题 + 新建分组 -->
    <div
      class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14 flex-shrink-0"
    >
      <span>表列表</span>
      <el-button
        v-if="store.currentProject && !store.readOnly"
        size="small"
        type="primary"
        link
        @click="openAddGroup"
      >
        + 分组
      </el-button>
    </div>

    <!-- 搜索 -->
    <div v-if="store.currentProject" class="px-8 py-6 flex-shrink-0">
      <el-input v-model="filterText" size="small" placeholder="搜索分组/表" clearable />
    </div>

    <!-- 树 -->
    <div class="flex-1 overflow-y-auto px-4 select-none">
      <el-empty v-if="!store.currentProject" description="未打开项目" :image-size="60" />
      <el-tree
        v-else
        ref="treeRef"
        :data="treeData"
        :props="treeProps"
        node-key="id"
        :filter-node-method="filterNode"
        :expand-on-click-node="false"
        :allow-drop="allowDrop"
        :draggable="!store.readOnly"
        default-expand-all
        @node-click="onNodeClick"
        @node-drop="onNodeDrop"
      >
        <template #default="{ data }">
          <div class="flex items-center w-full overflow-hidden">
            <span
              class="flex items-center gap-4 min-w-0 flex-1"
              :class="data.type === 'table' ? 'text-13' : 'font-bold text-13'"
            >
              <span class="flex-shrink-0">{{ data.type === "group" ? "📁" : "📄" }}</span>
              <span class="truncate min-w-0">{{ data.label }}</span>
            </span>
          </div>
        </template>
      </el-tree>
    </div>

    <!-- 新建分组对话框 -->
    <el-dialog draggable v-model="groupDialogVisible" title="新建分组" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="编码">
          <el-input v-model="groupCode" placeholder="如:order" />
        </el-form-item>
        <el-form-item label="中文名">
          <el-input v-model="groupName" placeholder="如:订单模块" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="groupDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmGroupDialog">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
/* el-tree-node white-space:nowrap + content 默认 overflow:visible,
   长 label 会溢出树边界。content 加 overflow:hidden 裁剪在树宽内。 */
:deep(.el-tree-node__content) {
  overflow: hidden;
}
</style>
