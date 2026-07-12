<script setup lang="ts">
// 表编辑器:表头 + 4 Tab(fields/index/java/json)。
// 通过 route param code 定位 store 里的表,直接编辑(Pinia 响应式)。
import { computed, ref } from "vue";
import { useProjectStore } from "@/stores/project";
import FieldsTab from "./table-editor/FieldsTab.vue";
import IndexTab from "./table-editor/IndexTab.vue";
import JavaTab from "./table-editor/JavaTab.vue";
import JsonTab from "./table-editor/JsonTab.vue";

const props = defineProps<{ code: string }>();
const store = useProjectStore();

const activeTab = ref("fields");

const table = computed(() =>
  store.currentProject?.tables.find((t) => t.code === props.code)
);

const groups = computed(() => store.currentProject?.groups ?? []);

function ensureIndexes(v: any) {
  if (table.value) table.value.indexes = v;
}
</script>

<template>
  <div v-if="table" class="h-full flex flex-col p-12 overflow-hidden">
    <!-- 表头 -->
    <div class="flex items-center gap-12 mb-12 flex-wrap flex-shrink-0">
      <span class="font-bold text-16">{{ table.code }}</span>
      <el-input v-model="table.name" size="small" placeholder="中文名" style="width: 140px" />
      <el-select v-model="table.group" size="small" placeholder="分组" style="width: 140px">
        <el-option v-for="g in groups" :key="g.code" :label="g.name" :value="g.code" />
      </el-select>
      <el-input
        v-model="table.comment"
        size="small"
        placeholder="表备注"
        style="width: 200px"
      />
    </div>

    <!-- 4 Tab -->
    <el-tabs v-model="activeTab" class="flex-1 flex flex-col table-tabs">
      <el-tab-pane label="字段" name="fields">
        <FieldsTab :fields="table.fields" />
      </el-tab-pane>
      <el-tab-pane label="索引" name="index">
        <IndexTab
          :indexes="table.indexes ?? []"
          :fields="table.fields"
          @update:indexes="ensureIndexes"
        />
      </el-tab-pane>
      <el-tab-pane label="Java" name="java" lazy>
        <JavaTab :table-code="table.code" />
      </el-tab-pane>
      <el-tab-pane label="JSON" name="json" lazy>
        <JsonTab :table-code="table.code" />
      </el-tab-pane>
    </el-tabs>
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
  overflow: hidden;
}
</style>
