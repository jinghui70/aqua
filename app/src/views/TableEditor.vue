<script setup lang="ts">
// 表编辑器:表头 + 4 Tab(fields/index/java/json)。
// 通过 route param id 定位 store 里的表,直接编辑(Pinia 响应式)。
import { computed, ref, watchEffect } from "vue";
import { ElMessage } from "element-plus";
import { useProjectStore } from "@/stores/project";
import FieldsTab from "./table-editor/FieldsTab.vue";
import IndexTab from "./table-editor/IndexTab.vue";
import DdlTab from "./table-editor/DdlTab.vue";
import JavaTab from "./table-editor/JavaTab.vue";
import JsonTab from "./table-editor/JsonTab.vue";

const props = defineProps<{ id: string }>();
const store = useProjectStore();

const activeTab = ref("fields");

const table = computed(() =>
  store.currentProject?.tables.find((t) => t.id === props.id)
);

// 保证 indexes 是 table 上的真实数组(否则 IndexTab 拿到临时数组,增删丢失)
watchEffect(() => {
  if (table.value && !table.value.indexes) table.value.indexes = [];
});

// 表基本信息编辑(code/name/comment;group 拖拽改)
const tableDialogVisible = ref(false);
const tableCode = ref("");
const tableName = ref("");
const tableComment = ref("");
function openEditTable() {
  if (!table.value) return;
  tableCode.value = table.value.code;
  tableName.value = table.value.name;
  tableComment.value = table.value.comment ?? "";
  tableDialogVisible.value = true;
}
function confirmTableDialog() {
  if (!table.value) return;
  const code = tableCode.value.trim().toUpperCase();
  const name = tableName.value.trim();
  if (!code || !name) { ElMessage.warning("code 和名称不能为空"); return; }
  const err = store.updateTable(table.value.id, code, name, tableComment.value.trim());
  if (err) { ElMessage.error(err); return; }
  ElMessage.success("表已更新");
  tableDialogVisible.value = false;
}
</script>

<template>
  <div v-if="table" class="h-full flex flex-col p-12 overflow-hidden">
    <!-- 表头 -->
    <div class="flex items-center gap-12 mb-12 flex-shrink-0">
      <span class="font-bold text-16">{{ table.code }}</span>
      <span class="text-gray-500 text-14">{{ table.name }}</span>
      <span v-if="table.comment" class="text-gray-400 text-12 truncate max-w-400">{{ table.comment }}</span>
      <el-button v-if="!store.readOnly" size="small" link @click="openEditTable">编辑</el-button>
    </div>

    <!-- 4 Tab -->
    <el-tabs v-model="activeTab" class="flex-1 flex flex-col table-tabs">
      <el-tab-pane label="字段" name="fields">
        <FieldsTab :fields="table.fields" :table-id="table.id" />
      </el-tab-pane>
      <el-tab-pane label="索引" name="index">
        <IndexTab :indexes="table.indexes ?? []" :fields="table.fields" :table-code="table.code" />
      </el-tab-pane>
      <el-tab-pane label="DDL" name="ddl" lazy>
        <DdlTab :table-code="table.code" :active="activeTab === 'ddl'" />
      </el-tab-pane>
      <el-tab-pane label="Java" name="java" lazy>
        <JavaTab :table-code="table.code" :active="activeTab === 'java'" />
      </el-tab-pane>
      <el-tab-pane label="JSON" name="json" lazy>
        <JsonTab :table-code="table.code" :active="activeTab === 'json'" />
      </el-tab-pane>
    </el-tabs>

    <!-- 表基本信息编辑弹框 -->
    <el-dialog draggable v-model="tableDialogVisible" title="编辑表" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="编码"><el-input v-model="tableCode" /></el-form-item>
        <el-form-item label="名称"><el-input v-model="tableName" /></el-form-item>
        <el-form-item label="备注"><el-input v-model="tableComment" type="textarea" :rows="2" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tableDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmTableDialog">保存</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="表不存在" />
</template>

<style scoped>
/* el-tabs 满高 + 内容区滚动控制。flex 链每层 min-height:0。 */
.table-tabs {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.table-tabs :deep(.el-tabs__header) {
  margin: 0;
  flex-shrink: 0;
}
.table-tabs :deep(.el-tabs__content) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.table-tabs :deep(.el-tab-pane) {
  height: 100%;
  /* padding-top 替代 margin-top:margin 不在 height 内,height:100%+margin 会溢出父容器,
     底部被 overflow:hidden 裁掉(编辑框底边框消失的根因)。padding 在 border-box 下含于 height。 */
  padding-top: 8px;
  overflow: hidden;
}
</style>
