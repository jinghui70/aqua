<script setup lang="ts">
// index Tab: 索引列表,name/字段(可排序+方向)/unique。
// indexes 由父组件保证为 table 上的真实数组引用,直接操作(Pinia 响应式)。
import type { Index, Field } from "@/types/schema";

const props = defineProps<{
  indexes: Index[];
  fields: Field[];
  /** 表 code,用于预览留空时自动生成的索引名 */
  tableCode: string;
}>();

function addIndex() {
  props.indexes.push({
    name: "",
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
/** 留空时预览自动生成的索引名 IDX_<TABLE>_<FIELDS>(复刻后端 auto_index_name)。 */
function autoName(idx: Index): string {
  const codes = idx.fields.map((f) => f.code).filter(Boolean).join("_");
  return `IDX_${props.tableCode}_${codes}`.toUpperCase();
}
</script>

<template>
  <div class="h-full overflow-auto">
    <div class="mb-12">
      <el-button size="small" type="primary" @click="addIndex">
        + 新增索引
      </el-button>
    </div>
    <el-table :data="indexes ?? []" border size="small" style="width: 100%">
      <el-table-column label="索引名" width="220">
        <template #default="{ row }">
          <el-input v-model="row.name" size="small" placeholder="留空自动生成" />
          <div v-if="!row.name" class="text-12 text-gray-400 mt-2">
            → {{ autoName(row) }}
          </div>
        </template>
      </el-table-column>
      <el-table-column label="字段" min-width="340">
        <template #default="{ row, $index }">
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
              <el-option v-for="c in fieldCodes()" :key="c" :label="c" :value="c" />
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
      </el-table-column>
      <el-table-column label="唯一" width="60" align="center">
        <template #default="{ row }">
          <el-checkbox v-model="row.unique" />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="70" align="center">
        <template #default="{ $index }">
          <el-button size="small" link type="danger" @click="removeIndex($index)">
            删
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
