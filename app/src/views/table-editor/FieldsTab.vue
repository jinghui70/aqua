<script setup lang="ts">
// fields Tab: 字段表格行内编辑 + 增删 + 拖拽排序 + 业务类型/自动生成单元格弹窗。
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import Sortable from "sortablejs";
import { DataType, type Field } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import { useClipboardStore } from "@/stores/clipboard";
import { useBuiltinStore } from "@/stores/builtin";
import { bizTypeSupports, applyDefaults } from "@/utils/bizType";
import BizTypeEditDialog from "./BizTypeEditDialog.vue";
import AutoGenEditDialog from "./AutoGenEditDialog.vue";

const props = defineProps<{ fields: Field[]; tableId: string }>();

const store = useProjectStore();
const clipboard = useClipboardStore();
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
    // (原生 DnD 的 mouseup 清理与 Vue 重渲染打架 -> 视图不更新 + 需二次点击)
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

// 业务类型/自动生成编辑弹窗
const bizTypeVisible = ref(false);
const bizTypeField = ref<Field | null>(null);
function openBizType(field: Field) {
  bizTypeField.value = field;
  bizTypeVisible.value = true;
}
const autoGenVisible = ref(false);
const autoGenField = ref<Field | null>(null);
function openAutoGen(field: Field) {
  autoGenField.value = field;
  autoGenVisible.value = true;
}

// 单元格点击: 业务类型/自动生成列整格可点开弹窗(其余列忽略)
function onCellClick(row: Field, column: { label?: string }) {
  if (store.readOnly) return;
  if (column.label === "业务类型") openBizType(row);
  else if (column.label === "自动生成策略") openAutoGen(row);
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

// autoGenerate 只读展示: 策略名(timing 由单元格图标体现,INSERT 不显,INSERT_UPDATE 显 update 图标)
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
    prop: "",
    code: "",
    name: "",
    dataType: DataType.Varchar,
    length: 32,
  });
  // 焦点定位到新行(末尾)的 code 输入框(优先 fixed 层,用户可见)
  nextTick(() => {
    const root = tableRef.value?.$el as HTMLElement | undefined;
    let inputs = root?.querySelectorAll<HTMLInputElement>(".el-table__fixed .code-cell input");
    if (!inputs?.length) inputs = root?.querySelectorAll<HTMLInputElement>(".code-cell input");
    inputs?.[inputs.length - 1]?.focus();
  });
}

// 清理不适用属性(§3.1): VARCHAR 仅 length,DECIMAL 仅 precision/scale,其余无
function cleanupDataType(field: Field, dt: DataType) {
  switch (dt) {
    case DataType.Varchar:
      field.precision = undefined;
      field.scale = undefined;
      break;
    case DataType.Decimal:
      field.length = undefined;
      break;
    default:
      field.length = undefined;
      field.precision = undefined;
      field.scale = undefined;
      break;
  }
}

// 应用新 dataType: 清理 + bizType 默认值 + 全局默认兜底(VARCHAR 32 / DECIMAL 10,4)
function applyDataType(field: Field, newDt: DataType) {
  field.dataType = newDt;
  cleanupDataType(field, newDt);
  const bt = field.bizType;
  if (bt && bt !== "Enum") {
    const def = bizTypeOptions.value.find((b) => b.bizType === bt);
    if (def) applyDefaults(field, def, newDt);
  }
  // bizType 未定义默认值时,填全局默认
  if (newDt === DataType.Varchar) {
    if (field.length == null) field.length = 32;
  } else if (newDt === DataType.Decimal) {
    if (field.precision == null) field.precision = 10;
    if (field.scale == null) field.scale = 4;
  }
}

// 切换类型 + bizType 兼容联动(§3.4):
// 兼容(或无 bizType) -> 应用新类型(含默认值)
// 不兼容(含 Enum 改非 VARCHAR) -> confirm 让用户决定:
//   确认 -> 清 bizType 后应用新类型;取消 -> 不动(dataType 保持旧值)
// 用 :model-value 受控,确认前不写 field.dataType,取消时 dataType 完全不变
async function onDataTypeChange(field: Field, newDt: DataType) {
  const bt = field.bizType;
  // 判断兼容性(field.dataType 仍是旧值)
  let incompatible = false;
  let bizName = "";
  if (bt) {
    if (bt === "Enum") {
      if (newDt !== DataType.Varchar) {
        incompatible = true;
        bizName = "Enum";
      }
    } else {
      const def = bizTypeOptions.value.find((b) => b.bizType === bt);
      if (def && !bizTypeSupports(def, newDt)) {
        incompatible = true;
        bizName = def.name;
      }
    }
  }
  if (incompatible) {
    try {
      await ElMessageBox.confirm(
        `数据类型 ${newDt} 与业务类型 ${bizName} 不兼容,切换将清除该业务类型。是否切换?`,
        "切换数据类型",
        { type: "warning", confirmButtonText: "切换", cancelButtonText: "取消" }
      );
      field.bizType = undefined;
      field.bizTypeData = undefined;
      if (bt === "Enum") field.enum = undefined;
      applyDataType(field, newDt);
    } catch {
      // 取消: dataType 保持旧值,不动
    }
    return;
  }
  // 兼容或无 bizType
  applyDataType(field, newDt);
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

// 多选选中(row 对象引用数组,删除时按引用定位 idx)
const selected = ref<Field[]>([]);
function onSelectionChange(rows: Field[]) {
  selected.value = rows;
}

const canCopy = computed(() => selected.value.length > 0);
const canPaste = computed(() => clipboard.has);
const canDelete = computed(() => selected.value.length > 0);

// 拷贝选中字段到全局剪贴板(深拷贝,跨表/跨项目可粘贴)
function copySelected() {
  if (!selected.value.length) return;
  clipboard.set(selected.value);
  ElMessage.success(`已拷贝 ${selected.value.length} 个字段`);
}

// 粘贴:同名 code 加 _n 后缀(2,3,...),prop 联动重新生成(蛇形转驼峰)
function paste() {
  if (!clipboard.has) return;
  const copied = clipboard.get();
  const used = new Set(props.fields.map((f) => f.code));
  for (const f of copied) {
    let code = f.code;
    let n = 2;
    while (used.has(code)) {
      code = `${f.code}_${n++}`;
    }
    used.add(code);
    f.code = code;
    const parts = code.split("_").filter(Boolean);
    if (parts.length) {
      f.prop =
        parts[0].toLowerCase() +
        parts.slice(1).map((p) => p[0].toUpperCase() + p.slice(1).toLowerCase()).join("");
    }
  }
  props.fields.push(...copied);
  ElMessage.success(`已粘贴 ${copied.length} 个字段`);
}

// 删除选中:按 idx 降序 splice 避免偏移 + 级联清理索引引用
function deleteSelected() {
  if (!selected.value.length) return;
  const idxs = selected.value
    .map((f) => props.fields.indexOf(f))
    .filter((i) => i >= 0)
    .sort((a, b) => b - a);
  for (const i of idxs) {
    const code = props.fields[i]?.code;
    props.fields.splice(i, 1);
    if (code) store.removeFieldFromIndexes(props.tableId, code);
  }
  ElMessage.success(`已删除 ${idxs.length} 个字段`);
  selected.value = [];
  tableRef.value?.clearSelection();
}
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="mb-12 flex-shrink-0 flex items-center">
      <el-button v-if="!store.readOnly" size="small" type="primary" @click="addField">
        + 新增字段
      </el-button>
      <el-button size="small" :disabled="!canCopy" @click="copySelected">
        <span class="i-mdi-content-copy w-16 h-16 mr-4" />
        拷贝
      </el-button>
      <el-button v-if="!store.readOnly" size="small" :disabled="!canPaste" @click="paste">
        <span class="i-mdi-content-paste w-16 h-16 mr-4" />
        粘贴
      </el-button>
      <el-button v-if="!store.readOnly" size="small" type="danger" :disabled="!canDelete" @click="deleteSelected">
        <span class="i-mdi-delete w-16 h-16 mr-4" />
        删除
      </el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-table
        ref="tableRef"
        :data="fields"
        :row-key="rowKey"
        border
        size="small"
        height="100%"
        class="select-none"
        style="width: 100%"
        @selection-change="onSelectionChange"
        @cell-click="onCellClick"
      >
      <el-table-column type="selection" width="36" fixed="left" />
      <el-table-column v-if="!store.readOnly" label="" width="36" align="center" key="drag" fixed="left">
        <template #default>
          <span class="drag-handle cursor-move text-gray-400 select-none">⣿</span>
        </template>
      </el-table-column>
      <el-table-column label="#" width="44" type="index" fixed="left" />
      <el-table-column label="编码" min-width="150" fixed="left">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.code }}</span>
          <el-input
            v-else
            v-model="row.code"
            class="code-cell"
            size="small"
            @focus="onCodeFocus(row)"
            @input="onCodeInput(row)"
            @change="onCodeChange(row)"
          />
        </template>
      </el-table-column>
      <el-table-column label="属性名" min-width="150">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.prop }}</span>
          <el-input v-else v-model="row.prop" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="名称" min-width="150">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.name }}</span>
          <el-input v-else v-model="row.name" size="small" />
        </template>
      </el-table-column>
      <el-table-column label="类型" width="200">
        <template #default="{ row }">
          <span v-if="store.readOnly" class="text-13">{{ row.dataType }}<template v-if="row.dataType === 'VARCHAR' && row.length">({{ row.length }})</template><template v-if="row.dataType === 'DECIMAL' && row.precision">({{ row.precision }},{{ row.scale ?? 0 }})</template></span>
          <div v-else class="flex items-center gap-4">
            <el-select :model-value="row.dataType" size="small" style="width: 100px" @change="(newDt: DataType) => onDataTypeChange(row, newDt)">
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
      <el-table-column label="主键" width="50" align="center">
        <template #default="{ row }">
          <span v-if="store.readOnly">{{ row.isKey ? "✓" : "" }}</span>
          <el-checkbox
            v-else
            :key="`key-${rowKey(row)}`"
            :model-value="row.isKey"
            :disabled="false"
            @change="(v: boolean) => onKeyChange(row, v)"
          />
        </template>
      </el-table-column>
      <el-table-column label="非空" width="50" align="center">
        <template #default="{ row }">
          <span v-if="store.readOnly">{{ row.notNull ? "✓" : "" }}</span>
          <el-checkbox v-else :key="`notnull-${rowKey(row)}`" v-model="row.notNull" :disabled="row.isKey" />
        </template>
      </el-table-column>
      <el-table-column label="业务类型" width="110" :class-name="store.readOnly ? '' : 'cursor-pointer'">
        <template #default="{ row }">
          <span :class="['text-13', store.readOnly ? '' : 'cursor-pointer text-blue-600 hover:underline']">{{ bizTypeLabel(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="自动生成策略" width="140" :class-name="store.readOnly ? '' : 'cursor-pointer'">
        <template #default="{ row }">
          <span :class="['text-13', store.readOnly ? '' : 'cursor-pointer text-blue-600 hover:underline']">
            <span v-if="row.autoGenerate?.timing === 'INSERT'" class="i-mdi-plus text-blue-500 inline-block w-14 h-14 align-middle mr-4" title="仅插入时生成" />
            <span v-else-if="row.autoGenerate?.timing === 'INSERT_UPDATE'" class="i-mdi-sync text-green-500 inline-block w-14 h-14 align-middle mr-4" title="插入和更新时生成" />
            {{ autoGenLabel(row) }}
          </span>
        </template>
      </el-table-column>
      <el-table-column label="默认值" min-width="90">
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
    </el-table>
    </div>
    <BizTypeEditDialog v-model="bizTypeVisible" :field="bizTypeField" />
    <AutoGenEditDialog v-model="autoGenVisible" :field="autoGenField" />
  </div>
</template>
