<script setup lang="ts">
// 左侧分组树:分组 > 表 两层。分组/表 CRUD,搜索,点表打开编辑标签。
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import type { Table } from "@/types/schema";

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

// 分组树数据:分组为父,表为子
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
        id: `table:${t.code}`,
        label: t.code,
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

// ===== 分组操作 =====
async function addGroup() {
  try {
    const { value } = await ElMessageBox.prompt(
      "分组 code 和名称(格式: code|名称,或只输 code)",
      "新建分组",
      { confirmButtonText: "创建", cancelButtonText: "取消", inputPlaceholder: "order|订单模块" }
    );
    if (!value) return;
    const [code, name] = value.includes("|") ? value.split("|") : [value, value];
    const err = store.addGroup(code.trim(), (name ?? code).trim());
    if (err) ElMessage.error(err);
    else ElMessage.success("分组已创建");
  } catch {
    /* 取消 */
  }
}

async function renameGroup(groupCode: string) {
  const g = store.currentProject?.groups.find((g) => g.code === groupCode);
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
async function addTable(groupCode: string) {
  try {
    const { value } = await ElMessageBox.prompt(
      "表名(大写蛇形,如 SYS_USER)",
      "新建表",
      { confirmButtonText: "创建", cancelButtonText: "取消", inputPlaceholder: "SYS_USER" }
    );
    if (!value) return;
    const code = value.trim().toUpperCase();
    const err = store.addTable(code, code, groupCode);
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("表已创建");
    const table = store.currentProject?.tables.find((t) => t.code === code);
    if (table) router.push(store.openTable(table));
  } catch {
    /* 取消 */
  }
}

async function renameTable(tableCode: string) {
  const t = store.currentProject?.tables.find((t) => t.code === tableCode);
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
        @click="addGroup"
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

    <!-- 树 -->
    <div class="flex-1 overflow-y-auto px-4">
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
        default-expand-all
        @node-click="onNodeClick"
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
                  @click.stop="addTable(data.groupCode)"
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
  </div>
</template>
