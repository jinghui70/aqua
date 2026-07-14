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
        id: `table:${t.code}`,
        label: `${t.code} (${t.name})`,
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

// ===== 分组操作 =====
// 新建分组对话框(code + 中文名)
const addGroupVisible = ref(false);
const addGroupCode = ref("");
const addGroupName = ref("");

function openAddGroup() {
  addGroupCode.value = "";
  addGroupName.value = "";
  addGroupVisible.value = true;
}
function confirmAddGroup() {
  const code = addGroupCode.value.trim();
  const name = addGroupName.value.trim();
  if (!code || !name) {
    ElMessage.warning("code 和名称不能为空");
    return;
  }
  const err = store.addGroup(code, name);
  if (err) {
    ElMessage.error(err);
    return;
  }
  addGroupVisible.value = false;
  ElMessage.success("分组已创建");
}

async function renameGroup(groupCode: string) {
  const g = store.currentProject?.groups.find((x) => x.code === groupCode);
  if (!g) return;
  try {
    const { value } = await ElMessageBox.prompt("分组名称", "重命名分组", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      inputValue: g.name,
    });
    if (value) store.renameGroup(groupCode, value.trim());
  } catch {
    /* 取消 */
  }
}

async function deleteGroup(groupCode: string) {
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
// 新建表对话框(code + 中文名)
const addTableVisible = ref(false);
const addTableGroup = ref("");
const addTableCode = ref("");
const addTableName = ref("");

function openAddTable(groupCode: string) {
  addTableGroup.value = groupCode;
  addTableCode.value = "";
  addTableName.value = "";
  addTableVisible.value = true;
}
function confirmAddTable() {
  const code = addTableCode.value.trim().toUpperCase();
  const name = addTableName.value.trim();
  if (!code || !name) {
    ElMessage.warning("code 和名称不能为空");
    return;
  }
  const err = store.addTable(code, name, addTableGroup.value);
  if (err) {
    ElMessage.error(err);
    return;
  }
  addTableVisible.value = false;
  ElMessage.success("表已创建");
  const table = store.currentProject?.tables.find((t) => t.code === code);
  if (table) router.push(store.openTable(table));
}

async function renameTable(tableCode: string) {
  const t = store.currentProject?.tables.find((x) => x.code === tableCode);
  if (!t) return;
  try {
    const { value } = await ElMessageBox.prompt("表中文名", "重命名表", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      inputValue: t.name,
    });
    if (value) store.renameTable(tableCode, value.trim());
  } catch {
    /* 取消 */
  }
}

async function deleteTable(tableCode: string) {
  try {
    await ElMessageBox.confirm(`确认删除表 ${tableCode}?`, "删除表", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
    store.deleteTable(tableCode);
    ElMessage.success("表已删除");
  } catch {
    /* 取消 */
  }
}

function onDuplicate(tableCode: string) {
  const newCode = store.duplicateTable(tableCode);
  if (!newCode) {
    ElMessage.error("复制失败");
    return;
  }
  const table = store.currentProject?.tables.find((t) => t.code === newCode);
  if (table) router.push(store.openTable(table));
  ElMessage.success("已复制");
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
          <div class="flex items-center justify-between w-full group pr-8">
            <span
              class="flex items-center gap-4"
              :class="data.type === 'table' ? 'text-13' : 'font-bold text-13'"
            >
              <span>{{ data.type === "group" ? "📁" : "📄" }}</span>
              {{ data.label }}
            </span>
            <!-- hover 操作 -->
            <span class="hidden group-hover:flex items-center gap-2">
              <template v-if="data.type === 'group'">
                <el-button
                  size="small"
                  link
                  type="primary"
                  @click.stop="openAddTable(data.groupCode)"
                  >+表</el-button
                >
                <el-button
                  size="small"
                  link
                  @click.stop="renameGroup(data.groupCode)"
                  >改</el-button
                >
                <el-button
                  size="small"
                  link
                  type="danger"
                  @click.stop="deleteGroup(data.groupCode)"
                  >删</el-button
                >
              </template>
              <template v-else>
                <el-button
                  size="small"
                  link
                  @click.stop="onDuplicate(data.tableCode)"
                  >复制</el-button
                >
                <el-button
                  size="small"
                  link
                  @click.stop="renameTable(data.tableCode)"
                  >改</el-button
                >
                <el-button
                  size="small"
                  link
                  type="danger"
                  @click.stop="deleteTable(data.tableCode)"
                  >删</el-button
                >
              </template>
            </span>
          </div>
        </template>
      </el-tree>
    </div>

    <!-- 新建分组对话框 -->
    <el-dialog v-model="addGroupVisible" title="新建分组" width="420px">
      <el-form label-width="80px">
        <el-form-item label="code">
          <el-input v-model="addGroupCode" placeholder="如:order" />
        </el-form-item>
        <el-form-item label="中文名">
          <el-input v-model="addGroupName" placeholder="如:订单模块" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addGroupVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmAddGroup">创建</el-button>
      </template>
    </el-dialog>

    <!-- 新建表对话框 -->
    <el-dialog v-model="addTableVisible" title="新建表" width="420px">
      <el-form label-width="80px">
        <el-form-item label="code">
          <el-input
            v-model="addTableCode"
            placeholder="大写蛇形,如:SYS_USER"
          />
        </el-form-item>
        <el-form-item label="中文名">
          <el-input v-model="addTableName" placeholder="如:用户表" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addTableVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmAddTable">创建</el-button>
      </template>
    </el-dialog>
  </div>
</template>
