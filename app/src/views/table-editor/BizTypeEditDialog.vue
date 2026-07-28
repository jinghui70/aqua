<script setup lang="ts">
// 业务类型编辑弹窗: bizType 下拉 + Enum 内联枚举 / bizTypeData 动态表单。
// 补齐行内表格放不下的业务类型属性。draft 副本编辑,保存写回。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { DataType, type Field, type InlineEnum, type BizTypeDefine } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";
import { bizTypeSupports, applyDefaults } from "@/utils/bizType";

const props = defineProps<{ modelValue: boolean; field: Field | null }>();
const emit = defineEmits<{ "update:modelValue": [boolean] }>();

const store = useProjectStore();
const builtin = useBuiltinStore();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

// 本地编辑副本(确认时写回)
const draft = ref<Field | null>(null);

const bizTypes = computed<BizTypeDefine[]>(() => [
  ...builtin.bizTypes,
  ...(store.currentProject?.bizTypes ?? []),
]);
// Enum 是特殊内置 bizType(§3.5),选中显示枚举配置;其他 bizType 显示 bizTypeData 表单
const isEnumBizType = computed(() => draft.value?.bizType === "Enum");
const currentBizType = computed(() =>
  bizTypes.value.find((b) => b.bizType === draft.value?.bizType)
);
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
  if (bizTypeDataFields.value.length === 1) {
    draft.value.bizTypeData = value;
  } else {
    const obj = { ...((draft.value.bizTypeData as Record<string, unknown>) ?? {}) };
    obj[field.name] = value;
    draft.value.bizTypeData = obj;
  }
}

// 保存前清理:空值或等于默认值的字段不输出(单属性 -> undefined,多属性 -> 删字段,空对象 -> undefined)
function cleanBizTypeData() {
  if (!draft.value?.bizTypeData) return;
  const fields = bizTypeDataFields.value;
  const isSkip = (v: unknown, def: unknown) =>
    v === "" || v === null || v === undefined || v === def;
  if (fields.length === 1) {
    if (isSkip(draft.value.bizTypeData, fields[0].default)) {
      draft.value.bizTypeData = undefined;
    }
  } else {
    const obj = { ...(draft.value.bizTypeData as Record<string, unknown>) };
    for (const f of fields) {
      if (isSkip(obj[f.name], f.default)) delete obj[f.name];
    }
    draft.value.bizTypeData = Object.keys(obj).length ? obj : undefined;
  }
}

// 按 bizType 定义初始化 bizTypeData:不预填默认值(默认值作 placeholder,用户输入才存;空/默认不保存)
function initBizTypeData(_def: BizTypeDefine): unknown {
  return undefined;
}

// bizType 切换:选 Enum 默认内联枚举并强制 VARCHAR;选普通 bizType 校正 dataType + 填默认值;离开清空
function onBizTypeChange(bizType: string | undefined) {
  if (!draft.value) return;
  draft.value.bizType = bizType;
  if (bizType === "Enum") {
    if (!draft.value.enum) {
      draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
    }
    draft.value.bizTypeData = undefined;
    draft.value.dataType = DataType.Varchar;
  } else if (bizType) {
    draft.value.enum = undefined;
    const def = bizTypes.value.find((b) => b.bizType === bizType);
    if (def) {
      let dt = draft.value.dataType;
      if (!bizTypeSupports(def, dt)) {
        dt = def.supportedDataTypes[0]?.dataType ?? dt;
        draft.value.dataType = dt;
      }
      applyDefaults(draft.value, def, dt);
      draft.value.bizTypeData = initBizTypeData(def) as Field["bizTypeData"];
    }
  } else {
    draft.value.enum = undefined;
  }
}

// dataType 下拉仅在该 bizType 支持多种类型时显示(Enum 强制 VARCHAR,单类型无选择)
const showDataTypeSelect = computed(
  () =>
    !!draft.value?.bizType &&
    draft.value.bizType !== "Enum" &&
    !!currentBizType.value &&
    currentBizType.value.supportedDataTypes.length > 1
);
// 弹窗内切 dataType: 清理不适用属性(§3.1) + 按 bizType 填默认长度精度
function onDataTypeChange() {
  if (!draft.value) return;
  switch (draft.value.dataType) {
    case DataType.Varchar:
      draft.value.precision = undefined;
      draft.value.scale = undefined;
      break;
    case DataType.Decimal:
      draft.value.length = undefined;
      break;
    default:
      draft.value.length = undefined;
      draft.value.precision = undefined;
      draft.value.scale = undefined;
      break;
  }
  const def = currentBizType.value;
  if (def) applyDefaults(draft.value, def, draft.value.dataType);
}

const inlineEnum = computed(() => draft.value?.enum ?? null);
function addInlineValue() {
  inlineEnum.value?.values.push({ id: "", name: "" });
}
function removeInlineValue(idx: number) {
  inlineEnum.value?.values.splice(idx, 1);
}

const COLORS = [
  "success", "error", "warning", "info", "primary", "danger",
  "red", "orange", "yellow", "green", "blue", "purple", "grey",
];

// 打开时重建 draft(深拷贝,取消不污染原对象;同字段再打开也重置为原始数据)
watch(visible, (v) => {
  if (v && props.field) {
    draft.value = JSON.parse(JSON.stringify(props.field));
  }
});

// ===== 保存 =====
function save() {
  if (!draft.value || !props.field) return;
  // enum 只支持 VARCHAR(onBizTypeChange 已强制,兜底)
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
  // 保存前清理 bizTypeData(空/默认值不输出)
  cleanBizTypeData();
  // 写回原字段(保持引用,Object.assign);本弹窗不编辑 code,无 code 级联
  Object.keys(props.field).forEach((k) => delete (props.field as any)[k]);
  Object.assign(props.field, draft.value);
  visible.value = false;
}
</script>

<template>
  <el-dialog v-model="visible" title="业务类型" width="720px" top="8vh" :close-on-click-modal="false">
    <div v-if="draft" class="flex flex-col gap-4" style="max-height: 70vh; overflow-y: auto">
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <el-form-item label="业务类型">
          <el-select
            :model-value="draft.bizType"
            clearable
            placeholder="-"
            style="width: 200px"
            @update:model-value="(v: string | undefined) => onBizTypeChange(v)"
          >
            <el-option label="Enum(枚举)" value="Enum" />
            <el-option v-for="b in bizTypes" :key="b.bizType" :label="b.name" :value="b.bizType" />
          </el-select>
          <el-select
            v-if="showDataTypeSelect"
            v-model="draft.dataType"
            size="small"
            style="width: 140px; margin-left: 12px"
            @change="onDataTypeChange"
          >
            <el-option v-for="s in (currentBizType?.supportedDataTypes ?? [])" :key="s.dataType" :label="s.dataType" :value="s.dataType" />
          </el-select>
        </el-form-item>

        <!-- bizType=Enum: 枚举特殊配置(内联)-->
        <template v-if="isEnumBizType">
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

        <!-- 其他 bizType: bizTypeData.fields 表单(两列)-->
        <template v-else>
          <div class="grid grid-cols-2 gap-x-24">
            <el-form-item
              v-for="bf in bizTypeDataFields"
              :key="bf.name"
              :label="bf.description || bf.name"
            >
              <el-input
                v-if="bf.type === 'string'"
                :model-value="getBizTypeDataValue(bf.name) as string"
                :placeholder="bf.default != null ? `默认:${bf.default}` : ''"
                @update:model-value="(v: string) => setBizTypeDataValue(bf, v)"
              />
              <el-input-number
                v-else
                :model-value="getBizTypeDataValue(bf.name) as number"
                :controls="false"
                :placeholder="bf.default != null ? `默认:${bf.default}` : ''"
                @update:model-value="(v: number | undefined) => setBizTypeDataValue(bf, v)"
              />
            </el-form-item>
          </div>
        </template>
      </el-form>
    </div>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :disabled="store.readOnly" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>
