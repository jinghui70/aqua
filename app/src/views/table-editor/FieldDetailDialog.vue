<script setup lang="ts">
// 字段完整编辑弹窗:补齐行内表格放不下的属性(autoGenerate.param / bizTypeData / enum)。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { DataType, type Field, type InlineEnum, type BizTypeDefine } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";

const props = defineProps<{ modelValue: boolean; field: Field | null; tableId: string }>();
const emit = defineEmits<{ "update:modelValue": [boolean] }>();

const store = useProjectStore();
const builtin = useBuiltinStore();
const dataTypes = Object.values(DataType);

const COLORS = [
  "success", "error", "warning", "info", "primary", "danger",
  "red", "orange", "yellow", "green", "blue", "purple", "grey",
];

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

// 本地编辑副本(确认时写回)
const draft = ref<Field | null>(null);

// ===== bizType =====
const bizTypes = computed<BizTypeDefine[]>(() => [
  ...builtin.bizTypes,
  ...(store.currentProject?.bizTypes ?? []),
]);
// Enum 是特殊内置 bizType(§3.5),选中显示枚举配置;其他 bizType 显示 bizTypeData 表单
const isEnumBizType = computed(() => draft.value?.bizType === "Enum");
const currentBizType = computed(() =>
  bizTypes.value.find((b) => b.bizType === draft.value?.bizType)
);
// bizType 的参数字段定义(用于动态表单)
const bizTypeDataFields = computed(
  () => currentBizType.value?.bizTypeData?.fields ?? []
);

// bizTypeData 值读写: 单 field 存值,多 field 存对象
function getBizTypeDataValue(fieldName: string): unknown {
  if (!draft.value) return undefined;
  const data = draft.value.bizTypeData;
  if (bizTypeDataFields.value.length === 1) return data;
  if (data && typeof data === "object") return (data as Record<string, unknown>)[fieldName];
  return undefined;
}
function setBizTypeDataValue(field: { name: string; default?: unknown }, value: unknown) {
  if (!draft.value) return;
  // 空值或等于默认值 -> 不存(输出 JSON 也不输出)
  const skip = value === "" || value === null || value === undefined || value === field.default;
  if (bizTypeDataFields.value.length === 1) {
    draft.value.bizTypeData = skip ? undefined : value;
  } else {
    const obj = { ...((draft.value.bizTypeData as Record<string, unknown>) ?? {}) };
    if (skip) delete obj[field.name];
    else obj[field.name] = value;
    draft.value.bizTypeData = Object.keys(obj).length ? obj : undefined;
  }
}

// ===== enum(无/内联,统一 InlineEnum)=====
type EnumMode = "none" | "inline";
const enumMode = ref<EnumMode>("none");
const isVarchar = computed(() => draft.value?.dataType === DataType.Varchar);

function syncEnumMode() {
  const e = draft.value?.enum;
  enumMode.value = e ? "inline" : "none";
}

// ===== 类型 ↔ bizType 联动(§3.4)=====
function bizTypeSupports(b: BizTypeDefine, dt: DataType): boolean {
  return b.supportedDataTypes.some((s) => s.dataType === dt);
}

// 选 bizType 后 dataType 下拉只显示其支持的类型(Enum 只支持 VARCHAR)
const availableDataTypes = computed<DataType[]>(() => {
  const bt = draft.value?.bizType;
  if (!bt) return dataTypes;
  if (bt === "Enum") return [DataType.Varchar];
  const def = bizTypes.value.find((b) => b.bizType === bt);
  return def ? def.supportedDataTypes.map((s) => s.dataType) : dataTypes;
});

// 选 dataType 后 bizType 下拉只显示支持该类型的业务类型
const availableBizTypes = computed<BizTypeDefine[]>(() => {
  const dt = draft.value?.dataType;
  if (!dt) return bizTypes.value;
  return bizTypes.value.filter((b) => bizTypeSupports(b, dt));
});

// 填充该 bizType 对指定 dataType 定义的默认 length/precision/scale
function applyDefaults(def: BizTypeDefine, dt: DataType) {
  if (!draft.value) return;
  const s = def.supportedDataTypes.find((x) => x.dataType === dt);
  if (!s) return;
  if (s.defaultLength != null) draft.value.length = s.defaultLength;
  if (s.defaultPrecision != null) draft.value.precision = s.defaultPrecision;
  if (s.defaultScale != null) draft.value.scale = s.defaultScale;
}

// 按 bizType 定义初始化 bizTypeData(单 field 存值,多 field 存对象;无 default 则不预填)
function initBizTypeData(def: BizTypeDefine): unknown {
  const fields = def.bizTypeData?.fields ?? [];
  if (!fields.length) return undefined;
  if (fields.length === 1) return fields[0].default;
  const obj: Record<string, unknown> = {};
  for (const f of fields) if (f.default !== undefined) obj[f.name] = f.default;
  return obj;
}

// dataType 切换:若当前 bizType 不支持新类型则清空,支持则填默认值
function onDataTypeChange(dt: DataType) {
  if (!draft.value) return;
  draft.value.dataType = dt;
  const bt = draft.value.bizType;
  if (bt === "Enum") {
    if (dt !== DataType.Varchar) onBizTypeChange(undefined); // Enum 只支持 VARCHAR
    return;
  }
  if (bt) {
    const def = bizTypes.value.find((b) => b.bizType === bt);
    if (def && !bizTypeSupports(def, dt)) onBizTypeChange(undefined);
    else if (def) applyDefaults(def, dt);
  }
}

// bizType 切换:选 Enum 默认内联枚举并强制 VARCHAR;选普通 bizType 校正 dataType + 填默认值;离开清空
function onBizTypeChange(bizType: string | undefined) {
  if (!draft.value) return;
  draft.value.bizType = bizType;
  if (bizType === "Enum") {
    if (!draft.value.enum) {
      draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
      enumMode.value = "inline";
    }
    draft.value.bizTypeData = undefined;
    draft.value.dataType = DataType.Varchar;
  } else if (bizType) {
    draft.value.enum = undefined;
    enumMode.value = "none";
    const def = bizTypes.value.find((b) => b.bizType === bizType);
    if (def) {
      let dt = draft.value.dataType;
      if (!bizTypeSupports(def, dt)) {
        dt = def.supportedDataTypes[0]?.dataType ?? dt;
        draft.value.dataType = dt;
      }
      applyDefaults(def, dt);
      draft.value.bizTypeData = initBizTypeData(def) as Field["bizTypeData"];
    }
  } else {
    draft.value.enum = undefined;
    enumMode.value = "none";
  }
}

// props.field 变化时重建 draft(声明在 enumMode 之后,避免 immediate watch 提前访问)
watch(
  () => props.field,
  (f) => {
    draft.value = f ? JSON.parse(JSON.stringify(f)) : null;
    syncEnumMode();
  },
  { immediate: true }
);

function onEnumModeChange(mode: EnumMode) {
  if (!draft.value) return;
  if (mode === "none") draft.value.enum = undefined;
  else draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
}

const inlineEnum = computed(() => draft.value?.enum ?? null);

function addInlineValue() {
  inlineEnum.value?.values.push({ id: "", name: "" });
}
function removeInlineValue(idx: number) {
  inlineEnum.value?.values.splice(idx, 1);
}

// code 大写 + 仅留合法字符(大写蛇形,不以数字开头)+ 蛇形转驼峰联动 prop
function onCodeInput() {
  if (!draft.value) return;
  draft.value.code = draft.value.code
    .toUpperCase()
    .replace(/[^A-Z0-9_]/g, "")
    .replace(/^[0-9]+/, "");
  const parts = draft.value.code.split("_").filter(Boolean);
  if (parts.length) {
    draft.value.prop =
      parts[0].toLowerCase() +
      parts
        .slice(1)
        .map((p) => p[0].toUpperCase() + p.slice(1).toLowerCase())
        .join("");
  }
}

// ===== 保存 =====
function save() {
  if (!draft.value || !props.field) return;
  // enum 只支持 VARCHAR
  if (draft.value.enum && draft.value.dataType !== DataType.Varchar) {
    ElMessage.error("enum 只支持 VARCHAR 类型");
    return;
  }
  // 内联 enum: hasCode=true 时每个枚举值 code 必填
  const e = draft.value.enum;
  if (e && e.hasCode) {
    if (e.values.some((v) => !v.code || !v.code.trim())) {
      ElMessage.error("hasCode 为 true 时,每个枚举值的 code 必填");
      return;
    }
  }
  // 写回原字段(保持引用,Object.assign);code 改名级联索引
  const oldCode = props.field.code;
  Object.keys(props.field).forEach((k) => delete (props.field as any)[k]);
  Object.assign(props.field, draft.value);
  if (oldCode !== draft.value.code) {
    store.renameFieldCode(props.tableId, oldCode, draft.value.code);
  }
  visible.value = false;
  ElMessage.success("已保存");
}
</script>

<template>
  <el-dialog v-model="visible" title="字段编辑" width="900px" top="6vh" :close-on-click-modal="false">
    <div v-if="draft" class="flex flex-col gap-4" style="max-height: 70vh; overflow-y: auto">
      <!-- 基本 -->
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <div class="grid grid-cols-2 gap-x-24">
          <el-form-item label="code">
            <el-input v-model="draft.code" @input="onCodeInput" />
          </el-form-item>
          <el-form-item label="prop">
            <el-input v-model="draft.prop" />
          </el-form-item>
          <el-form-item label="名称">
            <el-input v-model="draft.name" />
          </el-form-item>
          <el-form-item label="默认值">
            <el-input v-model="draft.defaultValue" placeholder="DDL DEFAULT 子句" />
          </el-form-item>
          <el-form-item label="类型">
            <el-select
              :model-value="draft.dataType"
              style="width: 160px"
              @update:model-value="(v: DataType) => onDataTypeChange(v)"
            >
              <el-option v-for="dt in availableDataTypes" :key="dt" :label="dt" :value="dt" />
            </el-select>
            <el-input-number
              v-if="draft.dataType === 'VARCHAR'"
              v-model="draft.length"
              :min="1"
              :controls="false"
              class="ml-8"
              style="width: 90px"
              placeholder="长度"
            />
            <template v-if="draft.dataType === 'DECIMAL'">
              <el-input-number v-model="draft.precision" :min="1" :controls="false" class="ml-8" style="width: 70px" placeholder="精度" />
              <el-input-number v-model="draft.scale" :min="0" :controls="false" class="ml-4" style="width: 70px" placeholder="小数" />
            </template>
          </el-form-item>
          <el-form-item label="约束">
            <el-checkbox v-model="draft.isKey" @change="draft.isKey && (draft.notNull = true)">主键</el-checkbox>
            <el-checkbox v-model="draft.notNull" :disabled="draft.isKey">非空</el-checkbox>
          </el-form-item>
        </div>
      </el-form>

      <!-- 自动生成 -->
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <el-form-item label="自动生成">
          <el-switch
            :model-value="!!draft.autoGenerate"
            @change="(v: any) => draft && (draft.autoGenerate = v ? { enabled: true, strategy: 'default', timing: 'INSERT' } : undefined)"
          />
        </el-form-item>
        <template v-if="draft.autoGenerate">
          <el-form-item label="策略">
            <el-select v-model="draft.autoGenerate.strategy" style="width: 160px">
              <el-option label="雪花id (default)" value="default" />
              <el-option label="当前时间 (now)" value="now" />
            </el-select>
          </el-form-item>
          <el-form-item label="参数">
            <el-input
              v-model="draft.autoGenerate.param"
              :placeholder="draft.autoGenerate.strategy === 'now' ? 'yyyy-MM-dd HH:mm:ss' : '如 USR_ 前缀'"
            />
          </el-form-item>
          <el-form-item label="时机">
            <el-select v-model="draft.autoGenerate.timing" style="width: 160px">
              <el-option label="INSERT" value="INSERT" />
              <el-option label="INSERT_UPDATE" value="INSERT_UPDATE" />
            </el-select>
          </el-form-item>
        </template>
      </el-form>

      <!-- 业务类型(Enum 是特殊 bizType)-->
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <el-form-item label="业务类型">
          <el-select
            :model-value="draft.bizType"
            clearable
            placeholder="-"
            style="width: 200px"
            @update:model-value="(v: string | undefined) => onBizTypeChange(v)"
          >
            <el-option v-if="isVarchar" label="Enum(枚举)" value="Enum" />
            <el-option v-for="b in availableBizTypes" :key="b.bizType" :label="b.name" :value="b.bizType" />
          </el-select>
        </el-form-item>

        <!-- bizType=Enum: 枚举特殊配置(无/内联)-->
        <template v-if="isEnumBizType">
          <!-- 选 Enum 自动内联枚举 -->
          <template v-if="inlineEnum">
            <el-form-item label="枚举名">
              <el-input v-model="inlineEnum.name" style="width: 200px" />
              <el-checkbox v-model="inlineEnum.hasCode" class="ml-12">hasCode</el-checkbox>
            </el-form-item>
            <el-form-item label="枚举值">
              <div class="w-full">
                <el-button size="small" :disabled="store.readOnly" @click="addInlineValue" class="mb-8">+ 添加值</el-button>
                <el-table :data="inlineEnum.values" border size="small">
                  <el-table-column label="id" width="110">
                    <template #default="{ row }"><el-input v-model="row.id" size="small" /></template>
                  </el-table-column>
                  <el-table-column label="名称" width="100">
                    <template #default="{ row }"><el-input v-model="row.name" size="small" /></template>
                  </el-table-column>
                  <el-table-column label="code" width="100">
                    <template #default="{ row }">
                      <el-input v-model="row.code" size="small" :placeholder="inlineEnum.hasCode ? '必填' : '-'" />
                    </template>
                  </el-table-column>
                  <el-table-column label="颜色" width="110">
                    <template #default="{ row }">
                      <el-select v-model="row.color" size="small" clearable placeholder="-">
                        <el-option v-for="c in COLORS" :key="c" :label="c" :value="c" />
                      </el-select>
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="50" align="center">
                    <template #default="{ $index }">
                      <el-button size="small" link type="danger" :disabled="store.readOnly" @click="removeInlineValue($index)">删</el-button>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </el-form-item>
          </template>
        </template>

        <!-- 其他 bizType: bizTypeData.fields 表单 -->
        <template v-else>
          <el-form-item
            v-for="bf in bizTypeDataFields"
            :key="bf.name"
            :label="bf.description || bf.name"
          >
            <el-input
              v-if="bf.type === 'string'"
              :model-value="getBizTypeDataValue(bf.name) as string"
              :placeholder="bf.default != null ? String(bf.default) : ''"
              @update:model-value="(v: string) => setBizTypeDataValue(bf, v)"
            />
            <el-input-number
              v-else
              :model-value="getBizTypeDataValue(bf.name) as number"
              :controls="false"
              :placeholder="bf.default != null ? String(bf.default) : ''"
              @update:model-value="(v: number | undefined) => setBizTypeDataValue(bf, v)"
            />
          </el-form-item>
        </template>
      </el-form>

      <!-- 备注 -->
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <el-form-item label="备注">
          <el-input v-model="draft.comment" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
    </div>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :disabled="store.readOnly" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>
