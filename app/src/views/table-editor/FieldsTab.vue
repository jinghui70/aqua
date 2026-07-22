<script setup lang="ts">
// fields Tab: 字段表格行内编辑 + 增删 + 拖拽排序 + 详情弹窗。
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import Sortable from "sortablejs";
import { DataType, type Field } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";
import FieldDetailDialog from "./FieldDetailDialog.vue";

const props = defineProps<{ fields: Field[]; tableId: string }>();

const store = useProjectStore();
const builtin = useBuiltinStore();

// 稳定 row-key: 给每个 field 对象分配递增 id(不污染 schema),
// 让 el-table 用 key diff,拖拽 splice 后 Vue 按 key 重排节点,与 Sortable 目标一致。
const keyMap = new WeakMap<Field, number>();
let keySeq = 0;
function rowKey(f: Field): number {
  let k = keyMap.get(f);
  if (k === undefined) {
    k = ++keySeq;
    keyMap.set(f, k);
  }
  return k;
}

// 拖拽排序: Sortable 挂在 el-table tbody,只允许手柄列触发
const tableRef = ref();
let sortableInst: Sortable | null = null;
onMounted(() => {
  const tbody = tableRef.value?.$el?.querySelector(
    ".el-table__body-wrapper tbody"
  );
  if (!tbody) return;
  sortableInst = Sortable.create(tbody, {
    handle: ".drag-handle",
    animation: 150,
    // 用 JS 模拟拖拽,绕开 HTML5 原生 DnD 与 el-table 自绘 DOM 的竞态
    // (原生 DnD 的 mouseup 清理与 Vue 重渲染打架 → 视图不更新 + 需二次点击)
    forceFallback: true,
    fallbackOnBody: true,
    disabled: store.readOnly,
    onEnd({ oldIndex, newIndex }) {
      if (oldIndex == null || newIndex == null || oldIndex === newIndex) return;
      // 延到下一 tick 改数据,让 Sortable 先完成本次拖拽的清理
      nextTick(() => {
        const [moved] = props.fields.splice(oldIndex, 1);
        props.fields.splice(newIndex, 0, moved);
      });
    },
  });
});
watch(() => store.readOnly, (ro) => sortableInst?.option("disabled", ro));

// 详情弹窗
const detailVisible = ref(false);
const detailField = ref<Field | null>(null);
function openDetail(field: Field) {
  detailField.value = field;
  detailVisible.value = true;
}
const dataTypes = Object.values(DataType);

// bizType 只读展示: 映射到名称(含内置)
const bizTypeOptions = computed(() => [
  ...builtin.bizTypes,
  ...(store.currentProject?.bizTypes ?? []),
]);
function bizTypeLabel(field: Field): string {
  if (!field.bizType) return "-";
  if (field.bizType === "Enum") return "Enum";
  return bizTypeOptions.value.find((b) => b.bizType === field.bizType)?.name ?? field.bizType;
}

// autoGenerate 只读展示: 策略名
function autoGenLabel(field: Field): string {
  const ag = field.autoGenerate;
  if (!ag) return "-";
  const strategies = [...builtin.autoGenStrategies, ...(store.currentProject?.autoGenStrategies ?? [])];
  return strategies.find((s) => s.code === ag.strategy)?.name ?? ag.strategy;
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
  const code = props.fields[idx]?.code;
  props.fields.splice(idx, 1);
  if (code) store.removeFieldFromIndexes(props.tableId, code);
}

// inline 改 code 前 focus 缓存旧值,用于级联索引
const oldCodeOnFocus = ref("");
function onCodeFocus(field: Field) {
  oldCodeOnFocus.value = field.code;
}

// code 输入实时:大写 + 仅留合法字符(大写蛇形,不以数字开头)+ 联动 prop(蛇形->驼峰)
function onCodeInput(field: Field) {
  field.code = field.code
    .toUpperCase()
    .replace(/[^A-Z0-9_]/g, "")
    .replace(/^[0-9]+/, "");
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

// code 失焦:级联索引(旧 code -> 新 code)
function onCodeChange(field: Field) {
  const oldCode = oldCodeOnFocus.value;
  oldCodeOnFocus.value = "";
  if (oldCode && oldCode !== field.code) {
    store.renameFieldCode(props.tableId, oldCode, field.code);
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
    <div class="mb-12 flex-shrink-0">
      <el-button v-if="!store.readOnly" size="small" type="primary" @click="addField">
        + 新增字段
      </el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-table ref="tableRef" :data="fields" :row-key="rowKey" border size="small" height="100%" class="select-none" style="width: 100%">
      <el-table-column v-if="!store.readOnly" label="" width="36" align="center" key="drag">
        <template #default>
          <span class="drag-handle cursor-move text-gray-400 select-none">⣿</span>
        </template>
      </el-table-column>
      <el-table-column label="#" width="44" type="index" />
      <el-table-column label="编码" width="150">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.code }}</span>
          <el-input
            v-else
            v-model="row.code"
            size="small"
            @focus="onCodeFocus(row)"
            @input="onCodeInput(row)"
            @change="onCodeChange(row)"
          />
        </template>
      </el-table-column>
      <el-table-column label="属性名" width="120">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.prop }}</span>
          <el-input v-else v-model="row.prop" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="名称" width="110">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.name }}</span>
          <el-input v-else v-model="row.name" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="类型" width="200">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.dataType }}<template v-if="row.dataType === 'VARCHAR' && row.length">({{ row.length }})</template><template v-if="row.dataType === 'DECIMAL' && row.precision">({{ row.precision }},{{ row.scale ?? 0 }})</template></span>
          <div v-else class="flex items-center gap-4">
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
          <span class="text-13">{{ bizTypeLabel(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="主键" width="50" align="center">
        <template #default="{ row }">
          <span v-if="store.readOnly">{{ row.isKey ? "✓" : "" }}</span>
          <el-checkbox
            v-else
            :model-value="row.isKey"
            @change="(v: boolean) => onKeyChange(row, v)"
          />
        </template>
      </el-table-column>
      <el-table-column label="非空" width="50" align="center">
        <template #default="{ row }">
          <span v-if="store.readOnly">{{ row.notNull ? "✓" : "" }}</span>
          <el-checkbox v-else v-model="row.notNull" :disabled="row.isKey" />
        </template>
      </el-table-column>
      <el-table-column label="自动生成" width="140">
        <template #default="{ row }">
          <span class="text-13">{{ autoGenLabel(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="默认值" width="110">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.defaultValue || "-" }}</span>
          <el-input v-else v-model="row.defaultValue" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="备注" min-width="120">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.comment || "-" }}</span>
          <el-input v-else v-model="row.comment" size="small" placeholder="-" />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120" align="center" fixed="right">
        <template #default="{ row, $index }">
          <el-button size="small" link type="primary" @click="openDetail(row)">详情</el-button>
          <el-button v-if="!store.readOnly" size="small" link @click="copyField($index)">复制</el-button>
          <el-button v-if="!store.readOnly" size="small" link type="danger" @click="removeField($index)">
            删
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    </div>
    <FieldDetailDialog v-model="detailVisible" :field="detailField" :table-id="tableId" />
  </div>
</template>
