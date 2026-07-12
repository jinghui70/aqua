<script setup lang="ts">
// fields Tab: 字段表格行内编辑 + 增删 + 排序 + 详情弹窗。
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import { DataType, type Field } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import FieldDetailDialog from "./FieldDetailDialog.vue";

const props = defineProps<{ fields: Field[] }>();

const store = useProjectStore();

// 详情弹窗
const detailVisible = ref(false);
const detailField = ref<Field | null>(null);
function openDetail(field: Field) {
  detailField.value = field;
  detailVisible.value = true;
}
const dataTypes = Object.values(DataType);

// bizType 下拉选项(来自项目定义的业务类型)
const bizTypeOptions = computed(() => store.currentProject?.bizTypes ?? []);

// autoGenerate 启用开关:开时初始化对象,关时清空
function toggleAutoGen(field: Field, enabled: boolean) {
  if (enabled) {
    field.autoGenerate = {
      enabled: true,
      strategy: "default",
      timing: "INSERT",
    };
  } else {
    field.autoGenerate = undefined;
  }
}

// 主键必然非空:选中主键时自动勾非空
function onKeyChange(field: Field, isKey: boolean) {
  field.isKey = isKey;
  if (isKey) field.notNull = true;
}

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
  // code 统一大写
  field.code = field.code.toUpperCase();
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
  <div class="h-full flex flex-col">
    <div class="mb-12 flex gap-8 flex-shrink-0">
      <el-button size="small" type="primary" @click="addField">
        + 新增字段
      </el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-table :data="fields" border size="small" height="100%" style="width: 100%">
      <el-table-column label="#" width="44" type="index" />
      <el-table-column label="code" width="150">
        <template #default="{ row }">
          <el-input
            v-model="row.code"
            size="small"
            @input="row.code = row.code.toUpperCase()"
            @change="onCodeChange(row)"
          />
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
      <el-table-column label="业务类型" width="130">
        <template #default="{ row }">
          <el-select
            v-model="row.bizType"
            size="small"
            clearable
            placeholder="-"
            style="width: 100%"
          >
            <el-option
              v-for="b in bizTypeOptions"
              :key="b.bizType"
              :label="b.name"
              :value="b.bizType"
            />
          </el-select>
        </template>
      </el-table-column>
      <el-table-column label="主键" width="50" align="center">
        <template #default="{ row }">
          <el-checkbox
            :model-value="row.isKey"
            @change="(v: boolean) => onKeyChange(row, v)"
          />
        </template>
      </el-table-column>
      <el-table-column label="非空" width="50" align="center">
        <template #default="{ row }">
          <el-checkbox v-model="row.notNull" :disabled="row.isKey" />
        </template>
      </el-table-column>
      <el-table-column label="自动生成" width="260">
        <template #default="{ row }">
          <div class="flex items-center gap-4">
            <el-switch
              :model-value="!!row.autoGenerate"
              size="small"
              @change="(v: string | number | boolean) => toggleAutoGen(row, !!v)"
            />
            <template v-if="row.autoGenerate">
              <el-select
                v-model="row.autoGenerate.strategy"
                size="small"
                style="width: 90px"
                placeholder="策略"
              >
                <el-option label="雪花id" value="default" />
                <el-option label="当前时间" value="now" />
              </el-select>
              <el-select
                v-model="row.autoGenerate.timing"
                size="small"
                style="width: 100px"
              >
                <el-option label="INSERT" value="INSERT" />
                <el-option label="INSERT_UPDATE" value="INSERT_UPDATE" />
              </el-select>
            </template>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="默认值" width="110">
        <template #default="{ row }">
          <el-input v-model="row.defaultValue" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="备注" min-width="120">
        <template #default="{ row }">
          <el-input v-model="row.comment" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="180" align="center">
        <template #default="{ row, $index }">
          <el-button size="small" link type="primary" @click="openDetail(row)">详情</el-button>
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
    <FieldDetailDialog v-model="detailVisible" :field="detailField" />
  </div>
</template>
