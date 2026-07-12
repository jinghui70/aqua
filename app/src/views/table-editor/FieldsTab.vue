<script setup lang="ts">
// fields Tab: 字段表格行内编辑 + 增删 + 排序。
import { ElMessage } from "element-plus";
import { DataType, type Field } from "@/types/schema";

const props = defineProps<{ fields: Field[] }>();

const dataTypes = Object.values(DataType);

function addField() {
  props.fields.push({
    prop: "newField",
    code: "NEW_FIELD",
    name: "新字段",
    dataType: DataType.Varchar,
    length: 64,
  });
}

function removeField(idx: number) {
  props.fields.splice(idx, 1);
}

function moveUp(idx: number) {
  if (idx === 0) return;
  const [f] = props.fields.splice(idx, 1);
  props.fields.splice(idx - 1, 0, f);
}

function moveDown(idx: number) {
  if (idx >= props.fields.length - 1) return;
  const [f] = props.fields.splice(idx, 1);
  props.fields.splice(idx + 1, 0, f);
}

// code 蛇形 -> prop 驼峰(输入 code 时自动填 prop)
function onCodeChange(field: Field) {
  const parts = field.code.split("_").filter(Boolean);
  if (parts.length) {
    field.prop =
      parts[0].toLowerCase() +
      parts
        .slice(1)
        .map((p) => p[0].toUpperCase() + p.slice(1).toLowerCase())
        .join("");
  }
}

function copyField(idx: number) {
  const src = props.fields[idx];
  const copy: Field = JSON.parse(JSON.stringify(src));
  copy.code = src.code + "_COPY";
  copy.prop = src.prop + "Copy";
  props.fields.splice(idx + 1, 0, copy);
  ElMessage.success("已复制字段");
}
</script>

<template>
  <div>
    <div class="mb-12 flex gap-8">
      <el-button size="small" type="primary" @click="addField">
        + 新增字段
      </el-button>
    </div>
    <el-table :data="fields" border size="small" style="width: 100%">
      <el-table-column label="#" width="44" type="index" />
      <el-table-column label="code" width="150">
        <template #default="{ row }">
          <el-input v-model="row.code" size="small" @change="onCodeChange(row)" />
        </template>
      </el-table-column>
      <el-table-column label="prop" width="120">
        <template #default="{ row }">
          <el-input v-model="row.prop" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="名称" width="110">
        <template #default="{ row }">
          <el-input v-model="row.name" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="类型" width="200">
        <template #default="{ row }">
          <div class="flex items-center gap-4">
            <el-select v-model="row.dataType" size="small" style="width: 100px">
              <el-option v-for="dt in dataTypes" :key="dt" :label="dt" :value="dt" />
            </el-select>
            <el-input-number
              v-if="row.dataType === 'VARCHAR'"
              v-model="row.length"
              size="small"
              :min="1"
              :controls="false"
              style="width: 70px"
              placeholder="长度"
            />
            <template v-if="row.dataType === 'DECIMAL'">
              <el-input-number
                v-model="row.precision"
                size="small"
                :min="1"
                :controls="false"
                style="width: 50px"
                placeholder="p"
              />
              <el-input-number
                v-model="row.scale"
                size="small"
                :min="0"
                :controls="false"
                style="width: 50px"
                placeholder="s"
              />
            </template>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="业务类型" width="110">
        <template #default="{ row }">
          <el-input v-model="row.bizType" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="主键" width="50" align="center">
        <template #default="{ row }">
          <el-checkbox v-model="row.isKey" />
        </template>
      </el-table-column>
      <el-table-column label="非空" width="50" align="center">
        <template #default="{ row }">
          <el-checkbox v-model="row.notNull" />
        </template>
      </el-table-column>
      <el-table-column label="备注" min-width="120">
        <template #default="{ row }">
          <el-input v-model="row.comment" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="150" align="center">
        <template #default="{ $index }">
          <el-button size="small" link @click="moveUp($index)">↑</el-button>
          <el-button size="small" link @click="moveDown($index)">↓</el-button>
          <el-button size="small" link @click="copyField($index)">复制</el-button>
          <el-button size="small" link type="danger" @click="removeField($index)">
            删
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>
