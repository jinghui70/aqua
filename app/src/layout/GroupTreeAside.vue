<script setup lang="ts">
// 左侧分组树:分组 > 表 两层。分组/表 CRUD,搜索,点表打开编辑标签,拖拽排序/移动,表复制。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
const router = useRouter();

const filterText = ref("");
const treeRef = ref();

interface TreeNode {
  /** 唯一 key */
  id: string;
  label: string;
  /** group | table */
  type: "group" | "table";
  /** 分组 code(table 节点为所属分组) */
  groupCode: string;
  /** table 节点的表 code */
  tableCode?: string;
  children?: TreeNode[];
}

// 分组树数据:分组为父,表为子。分组只显中文名,表显 `code (中文名)`。
const treeData = computed<TreeNode[]>(() => {
  const p = store.currentProject;
  if (!p) return [];
  return p.groups.map((g) => ({
    id: `group:${g.code}`,
    label: g.name,
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

// 点击节点:表 -> 打开编辑标签
function onNodeClick(data: TreeNode) {
  if (data.type === "table" && data.tableCode) {
    const table = store.currentProject?.tables.find(
      (t) => t.code === data.tableCode
    );
    if (table) {
      const path = store.openTable(table);
      router.push(path);
    }
  }
}

// ===== 拖拽:层级约束 + 落位写回 store =====
// el-tree 回调参数为内部 Node 类型,这里只取 data。
/* eslint-disable @typescript-eslint/no-explicit-any */
function allowDrop(draggingNode: any, dropNode: any, type: string): boolean {
  const drag = draggingNode.data as TreeNode;
  const drop = dropNode.data as TreeNode;
  if (drag.type === "group") {
    // 分组只能排在分组旁(before/after),不能进分组(inner)
    return type !== "inner" && drop.type === "group";
  }
  // 表:inner 进分组;before/after 排在表旁
  if (type === "inner") return drop.type === "group";
  return drop.type === "table";
}

function onNodeDrop(draggingNode: any, dropNode: any, dropType: string) {
  const drag = draggingNode.data as TreeNode;
  const drop = dropNode.data as TreeNode;
  if (drag.type === "group") {
    store.reorderGroups(
      drag.groupCode,
      drop.groupCode,
      dropType as "before" | "after"
    );
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

// ===== hover 操作栏(Teleport 到 body + fixed,不受树容器 overflow 裁剪) =====
const hoverNode = ref<TreeNode | null>(null);
const hoverBarStyle = ref<Record<string, string>>({});
let leaveTimer: ReturnType<typeof setTimeout> | null = null;

function onNodeEnter(data: TreeNode, ev: MouseEvent) {
  if (leaveTimer) {
    clearTimeout(leaveTimer);
    leaveTimer = null;
  }
  const rect = (ev.currentTarget as HTMLElement).getBoundingClientRect();
  hoverNode.value = data;
  // right 用视口宽度减节点右边缘,使按钮右边缘对齐节点右边缘
  hoverBarStyle.value = {
    top: `${rect.top}px`,
    right: `${window.innerWidth - rect.right}px`,
  };
}
function onNodeLeave() {
  leaveTimer = setTimeout(() => {
    hoverNode.value = null;
  }, 120);
}
function onBarEnter() {
  if (leaveTimer) {
    clearTimeout(leaveTimer);
    leaveTimer = null;
  }
}
function onBarLeave() {
  hoverNode.value = null;
}

// ===== 分组操作 =====
// 新建/编辑共用对话框:code 编辑时只读(业务键,被表引用,不可改)
const groupDialogVisible = ref(false);
const groupDialogMode = ref<"add" | "edit">("add");
const groupEditingCode = ref("");
const groupCode = ref("");
const groupName = ref("");

function openAddGroup() {
  groupDialogMode.value = "add";
  groupEditingCode.value = "";
  groupCode.value = "";
  groupName.value = "";
  groupDialogVisible.value = true;
}
function openEditGroup(code?: string) {
  if (!code) return;
  const g = store.currentProject?.groups.find((x) => x.code === code);
  if (!g) return;
  groupDialogMode.value = "edit";
  groupEditingCode.value = code;
  groupCode.value = g.code;
  groupName.value = g.name;
  groupDialogVisible.value = true;
}
function confirmGroupDialog() {
  const code = groupCode.value.trim();
  const name = groupName.value.trim();
  if (!code || !name) {
    ElMessage.warning("code 和名称不能为空");
    return;
  }
  if (groupDialogMode.value === "add") {
    const err = store.addGroup(code, name);
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("分组已创建");
  } else {
    const err = store.updateGroup(groupEditingCode.value, code, name);
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("分组已更新");
  }
  groupDialogVisible.value = false;
}

async function deleteGroup(groupCode?: string) {
  if (!groupCode) return;
  try {
    await ElMessageBox.confirm(`确认删除分组 ${groupCode}?`, "删除分组", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
    const err = store.deleteGroup(groupCode);
    if (err) ElMessage.error(err);
    else ElMessage.success("分组已删除");
  } catch {
    /* 取消 */
  }
}

// ===== 表操作 =====
// 新建/编辑共用对话框:code 可编辑;归属通过拖拽改,对话框内不出现
const tableDialogVisible = ref(false);
const tableDialogMode = ref<"add" | "edit">("add");
const tableEditingId = ref("");
const addTableGroup = ref("");
const tableCode = ref("");
const tableName = ref("");
const tableComment = ref("");

function openAddTable(groupCode?: string) {
  if (!groupCode) return;
  tableDialogMode.value = "add";
  addTableGroup.value = groupCode;
  tableEditingId.value = "";
  tableCode.value = "";
  tableName.value = "";
  tableComment.value = "";
  tableDialogVisible.value = true;
}
function openEditTable(code?: string) {
  if (!code) return;
  const t = store.currentProject?.tables.find((x) => x.code === code);
  if (!t) return;
  tableDialogMode.value = "edit";
  tableEditingId.value = t.id;
  tableCode.value = t.code;
  tableName.value = t.name;
  tableComment.value = t.comment ?? "";
  tableDialogVisible.value = true;
}
function confirmTableDialog() {
  const code = tableCode.value.trim().toUpperCase();
  const name = tableName.value.trim();
  if (!code || !name) {
    ElMessage.warning("code 和名称不能为空");
    return;
  }
  if (tableDialogMode.value === "add") {
    const err = store.addTable(code, name, addTableGroup.value, tableComment.value.trim());
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("表已创建");
    tableDialogVisible.value = false;
    const table = store.currentProject?.tables.find((t) => t.code === code);
    if (table) router.push(store.openTable(table));
  } else {
    const err = store.updateTable(tableEditingId.value, code, name, tableComment.value.trim());
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("表已更新");
    tableDialogVisible.value = false;
  }
}

async function deleteTable(tableCode?: string) {
  if (!tableCode) return;
  try {
    await ElMessageBox.confirm(`确认删除表 ${tableCode}?`, "删除表", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
    const nextPath = store.deleteTable(tableCode);
    if (nextPath) router.push(nextPath);
    ElMessage.success("表已删除");
  } catch {
    /* 取消 */
  }
}

function onDuplicate(tableCode?: string) {
  if (!tableCode) {
    ElMessage.error("表编码缺失");
    return;
  }
  const newCode = store.duplicateTable(tableCode);
  if (!newCode) {
    ElMessage.error("复制失败");
    return;
  }
  const table = store.currentProject?.tables.find((t) => t.code === newCode);
  if (table) router.push(store.openTable(table));
  ElMessage.success(`已复制为 ${newCode}`);
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
        v-if="store.currentProject"
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
      <el-input
        v-model="filterText"
        size="small"
        placeholder="搜索分组/表"
        clearable
      />
    </div>

    <!-- 树(select-none 防止拖拽时选中文字) -->
    <div class="flex-1 overflow-y-auto px-4 select-none">
      <el-empty
        v-if="!store.currentProject"
        description="未打开项目"
        :image-size="60"
      />
      <el-tree
        v-else
        ref="treeRef"
        :data="treeData"
        :props="treeProps"
        node-key="id"
        :filter-node-method="filterNode"
        :expand-on-click-node="false"
        :allow-drop="allowDrop"
        draggable
        default-expand-all
        @node-click="onNodeClick"
        @node-drop="onNodeDrop"
      >
        <template #default="{ data }">
          <div
            class="flex items-center w-full overflow-hidden"
            @mouseenter="onNodeEnter(data, $event)"
            @mouseleave="onNodeLeave"
          >
            <span
              class="flex items-center gap-4 min-w-0 flex-1"
              :class="data.type === 'table' ? 'text-13' : 'font-bold text-13'"
            >
              <span class="flex-shrink-0">{{
                data.type === "group" ? "📁" : "📄"
              }}</span>
              <span class="truncate min-w-0">{{ data.label }}</span>
            </span>
          </div>
        </template>
      </el-tree>
    </div>

    <!-- 新建/编辑分组对话框 -->
    <el-dialog
      v-model="groupDialogVisible"
      :title="groupDialogMode === 'add' ? '新建分组' : '编辑分组'"
      width="420px"
    >
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
        <el-button type="primary" @click="confirmGroupDialog">{{
          groupDialogMode === "add" ? "创建" : "保存"
        }}</el-button>
      </template>
    </el-dialog>

    <!-- 新建/编辑表对话框 -->
    <el-dialog
      v-model="tableDialogVisible"
      :title="tableDialogMode === 'add' ? '新建表' : '编辑表'"
      width="420px"
    >
      <el-form label-width="80px">
        <el-form-item label="编码">
          <el-input v-model="tableCode" placeholder="大写蛇形,如:SYS_USER" />
        </el-form-item>
        <el-form-item label="中文名">
          <el-input v-model="tableName" placeholder="如:用户表" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="tableComment" type="textarea" :rows="2" placeholder="表说明(输出设计文档用)" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tableDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmTableDialog">{{
          tableDialogMode === "add" ? "创建" : "保存"
        }}</el-button>
      </template>
    </el-dialog>

    <!-- hover 操作栏:Teleport 到 body + fixed,脱离树 DOM,树窄时也不被裁 -->
    <Teleport to="body">
      <div
        v-if="hoverNode"
        class="hover-bar fixed flex flex-col items-center gap-1 bg-white border border-gray-200 rounded-4 shadow px-4 py-4 z-50"
        :style="hoverBarStyle"
        @mouseenter="onBarEnter"
        @mouseleave="onBarLeave"
      >
        <template v-if="hoverNode?.type === 'group'">
          <el-button
            size="small"
            link
            type="primary"
            @click="openAddTable(hoverNode?.groupCode)"
            >+表</el-button
          >
          <el-button size="small" link @click="openEditGroup(hoverNode?.groupCode)"
            >修改</el-button
          >
          <el-button
            size="small"
            link
            type="danger"
            @click="deleteGroup(hoverNode?.groupCode)"
            >删除</el-button
          >
        </template>
        <template v-else>
          <el-button size="small" link @click="onDuplicate(hoverNode?.tableCode)"
            >复制</el-button
          >
          <el-button size="small" link @click="openEditTable(hoverNode?.tableCode)"
            >修改</el-button
          >
          <el-button
            size="small"
            link
            type="danger"
            @click="deleteTable(hoverNode?.tableCode)"
            >删除</el-button
          >
        </template>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
/* el-tree-node white-space:nowrap + content 默认 overflow:visible,
   长 label 会溢出树边界。content 加 overflow:hidden 裁剪在树宽内。 */
:deep(.el-tree-node__content) {
  overflow: hidden;
}
/* hover 操作栏:按钮等宽居中,纵向整齐;清掉 el-button 相邻默认 margin-left 的错位 */
.hover-bar :deep(.el-button) {
  width: 56px;
  justify-content: center;
  margin-left: 0;
}
</style>
