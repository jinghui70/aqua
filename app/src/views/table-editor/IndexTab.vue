<script setup lang="ts">
// index Tab: 索引表格,name/fields(多选表内字段)/unique。
// indexes 由父组件保证为 table 上的真实数组引用,直接操作(Pinia 响应式)。
import type { Index, Field } from "@/types/schema";

const props = defineProps<{ indexes: Index[]; fields: Field[] }>();

function addIndex() {
  props.indexes.push({ name: "", fields: [], unique: false });
}

function removeIndex(idx: number) {
  props.indexes.splice(idx, 1);
}

const fieldCodes = () => props.fields.map((f) => f.code);
</script>

<template>
  <div class="h-full overflow-auto">
    <div class="mb-12">
      <el-button size="small" type="primary" @click="addIndex">
        + 新增索引
      </el-button>
    </div>
    <el-table :data="indexes ?? []" border size="small" style="width: 100%">
      <el-table-column label="索引名" width="200">
        <template #default="{ row }">
          <el-input v-model="row.name" size="small" placeholder="留空自动生成" />
        </template>
      </el-table-column>
      <el-table-column label="字段" min-width="240">
        <template #default="{ row }">
          <el-select
            v-model="row.fields"
            multiple
            size="small"
            style="width: 100%"
            placeholder="选择字段"
          >
            <el-option
              v-for="c in fieldCodes()"
              :key="c"
              :label="c"
              :value="c"
            />
          </el-select>
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
    <el-empty
      v-if="!indexes || !indexes.length"
      description="暂无索引"
      :image-size="50"
    />
  </div>
</template>
