<script setup lang="ts">
// index Tab: 索引列表,name/字段(可排序+方向)/unique。
// indexes 由父组件保证为 table 上的真实数组引用,直接操作(Pinia 响应式)。
import { nextTick, onMounted, ref, watch } from "vue";
import type { Index, IndexField, Field } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import Sortable from "sortablejs";

const store = useProjectStore();

const props = defineProps<{
  indexes: Index[];
  fields: Field[];
  /** 表 code,用于预览留空时自动生成的索引名 */
  tableCode: string;
}>();

/** 下一个索引序号: 扫描已有 IDX_<TABLE>_<N> 取最大 N +1,无则 1。 */
function nextIndexSeq(): number {
  const prefix = `IDX_${props.tableCode}_`.toUpperCase();
  let max = 0;
  for (const idx of props.indexes) {
    const name = (idx.name ?? "").toUpperCase();
    if (name.startsWith(prefix)) {
      const n = parseInt(name.slice(prefix.length), 10);
      if (!isNaN(n) && n > max) max = n;
    }
  }
  return max + 1;
}
function addIndex() {
  const seq = nextIndexSeq();
  props.indexes.push({
    name: `IDX_${props.tableCode}_${seq}`.toUpperCase(),
    fields: [{ code: "", direction: "ASC" }],
    unique: false,
  });
}
function removeIndex(idx: number) {
  props.indexes.splice(idx, 1);
}
function addField(idx: number) {
  props.indexes[idx].fields.push({ code: "", direction: "ASC" });
}
function removeField(idx: number, fi: number) {
  props.indexes[idx].fields.splice(fi, 1);
}
function moveField(idx: number, fi: number, dir: -1 | 1) {
  const fields = props.indexes[idx].fields;
  const ni = fi + dir;
  if (ni < 0 || ni >= fields.length) return;
  [fields[fi], fields[ni]] = [fields[ni], fields[fi]];
}
const fieldCodes = () => props.fields.map((f) => f.code);
/** 只读展示:索引字段列表文本(如 USER_ID, NAME DESC) */
function indexFieldsText(idx: Index): string {
  return (
    idx.fields
      .map((f) => f.code + (f.direction === "DESC" ? " DESC" : ""))
      .filter(Boolean)
      .join(", ") || "-"
  );
}

// 拖拽排序索引顺序(Sortable 挂 el-table tbody)
const tableRef = ref();
let sortableInst: Sortable | null = null;
onMounted(() => {
  const tbody = tableRef.value?.$el?.querySelector(".el-table__body-wrapper tbody");
  if (!tbody) return;
  sortableInst = Sortable.create(tbody, {
    handle: ".drag-handle",
    animation: 150,
    forceFallback: true,
    fallbackOnBody: true,
    disabled: store.readOnly,
    onEnd({ oldIndex, newIndex }) {
      if (oldIndex == null || newIndex == null || oldIndex === newIndex) return;
      nextTick(() => {
        const [moved] = props.indexes.splice(oldIndex, 1);
        props.indexes.splice(newIndex, 0, moved);
      });
    },
  });
});
watch(() => store.readOnly, (ro) => sortableInst?.option("disabled", ro));
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="mb-12 flex-shrink-0">
      <el-button v-if="!store.readOnly" size="small" type="primary" @click="addIndex">
        + 新增索引
      </el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-table ref="tableRef" :data="indexes ?? []" border size="small" height="100%" style="width: 100%">
      <el-table-column v-if="!store.readOnly" label="" width="36" align="center" key="drag">
        <template #default>
          <span class="drag-handle cursor-move text-gray-400 select-none">⣿</span>
        </template>
      </el-table-column>
      <el-table-column label="索引名" width="220">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.name || "-" }}</span>
          <el-input v-else v-model="row.name" size="small" placeholder="必填" />
        </template>
      </el-table-column>
      <el-table-column label="字段" min-width="340">
        <template #default="{ row, $index }">
          <span v-if="store.readOnly" class="text-13">{{ indexFieldsText(row) }}</span>
          <template v-else>
            <div
              v-for="(f, fi) in row.fields"
              :key="fi"
              class="flex items-center gap-2 mb-4"
            >
              <el-select
                v-model="f.code"
                size="small"
                filterable
                placeholder="字段"
                style="width: 140px"
              >
                <el-option
                  v-for="c in fieldCodes().filter((c) => c && !row.fields.some((o: IndexField, oi: number) => oi !== fi && o.code === c))"
                  :key="c"
                  :label="c"
                  :value="c"
                />
              </el-select>
              <el-select v-model="f.direction" size="small" style="width: 90px">
                <el-option label="ASC" value="ASC" />
                <el-option label="DESC" value="DESC" />
              </el-select>
              <el-button size="small" link :disabled="fi === 0" @click="moveField($index, fi, -1)">↑</el-button>
              <el-button size="small" link :disabled="fi === row.fields.length - 1" @click="moveField($index, fi, 1)">↓</el-button>
              <el-button size="small" link type="danger" @click="removeField($index, fi)">删</el-button>
            </div>
            <el-button size="small" @click="addField($index)">+ 字段</el-button>
          </template>
        </template>
      </el-table-column>
      <el-table-column label="唯一" width="60" align="center">
        <template #default="{ row }">
          <span v-if="store.readOnly">{{ row.unique ? "✓" : "" }}</span>
          <el-checkbox v-else v-model="row.unique" />
        </template>
      </el-table-column>
      <el-table-column v-if="!store.readOnly" label="操作" width="70" align="center">
        <template #default="{ $index }">
          <el-button size="small" link type="danger" @click="removeIndex($index)">
            删
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    </div>
  </div>
</template>
