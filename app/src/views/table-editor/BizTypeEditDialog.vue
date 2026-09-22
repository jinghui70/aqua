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

// ===== 枚举来源:新建定义 / 引用已有 =====
const enumSource = ref<"define" | "ref">("define");
// 引用级联选中值
const refTable = ref("");
const refField = ref("");

// 含至少一个"定义方枚举"字段的表(定义方 = 字段有 enum 且 !enum.ref)
const enumRefTables = computed(() =>
  (store.currentProject?.tables ?? []).filter((t) =>
    t.fields.some((f) => f.enum && !f.enum.ref)
  )
);
// 选中表内的定义方枚举字段
const enumRefFields = computed(() => {
  const t = (store.currentProject?.tables ?? []).find((x) => x.code === refTable.value);
  if (!t) return [];
  return t.fields.filter((f) => f.enum && !f.enum.ref);
});
// 引用目标"字段名 → 类名"label:类名来自目标字段 prop 派生 PascalCase
function propToPascal(p: string): string {
  return p ? p.charAt(0).toUpperCase() + p.slice(1) : "";
}
function enumClassName(f: Field): string {
  return f.enum?.className || propToPascal(f.prop);
}

// 定义模式默认值:名称=字段名称,Java 类名=prop 转 PascalCase(仅在为空时填入)
function ensureEnumDefaults() {
  const e = draft.value?.enum;
  if (e && !e.ref) {
    if (!e.name) e.name = draft.value?.name ?? "";
    if (!e.className) e.className = propToPascal(draft.value?.prop ?? "");
  }
}

// 引用目标(只读):被引用字段的完整信息
const refTarget = computed(() => {
  const t = (store.currentProject?.tables ?? []).find((x) => x.code === refTable.value);
  const f = t?.fields.find((x) => x.prop === refField.value);
  if (!t || !f) return null;
  return { table: t, field: f };
});

// 来源切换:ref -> define 重建定义;define -> ref 清空定义部分
function onEnumSourceChange(src: "define" | "ref") {
  if (!draft.value) return;
  if (src === "ref") {
    refTable.value = "";
    refField.value = "";
    if (draft.value.enum) {
      draft.value.enum.ref = undefined;
      draft.value.enum.name = "";
      draft.value.enum.hasCode = false;
      draft.value.enum.values = [];
    }
  } else {
    if (!draft.value.enum) {
      draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
    }
    draft.value.enum.ref = undefined;
    ensureEnumDefaults();
  }
  enumSource.value = src;
}

// 级联选择:选表后默认选中该表第一个枚举字段;选字段后写 enum.ref 并反写当前字段
function onRefTableChange() {
  refField.value = "";
  if (draft.value?.enum) draft.value.enum.ref = undefined;
  // 默认选中该表第一个枚举字段
  const first = enumRefFields.value[0];
  if (first) {
    refField.value = first.prop;
    applyRefTarget();
  }
}
function onRefFieldChange() {
  if (!refTable.value || !refField.value) return;
  applyRefTarget();
}
// 写 enum.ref + 反写当前字段。编码/属性名/名称仅在当前为空时带入(不覆盖已填);
// 类型/长度强制对齐目标(后端一致性校验要求,空与非空均同步)。
function applyRefTarget() {
  if (!draft.value?.enum || !refTable.value || !refField.value) return;
  const t = (store.currentProject?.tables ?? []).find((x) => x.code === refTable.value);
  const target = t?.fields.find((x) => x.prop === refField.value);
  if (!target) return;
  draft.value.enum.ref = { code: refTable.value, prop: refField.value };
  draft.value.enum.name = target.enum?.name ?? "";
  if (!draft.value.code) draft.value.code = target.code;
  if (!draft.value.prop) draft.value.prop = target.prop;
  if (!draft.value.name) draft.value.name = target.name;
  // 对齐目标类型时清理不适用属性(§3.1),避免旧类型 length/precision 残留
  cleanupDataType(draft.value, target.dataType);
  draft.value.dataType = target.dataType;
  draft.value.length = target.length;
}

// hasCode 联动:取消勾选时清空各行 code(数据不留脏值,列表同步隐藏 code 列)
function onHasCodeChange(val: boolean | string | number) {
  if (!val && inlineEnum.value) {
    inlineEnum.value.values.forEach((v) => (v.code = undefined));
  }
}

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
  const prevBizType = draft.value.bizType;
  draft.value.bizType = bizType;
  enumSource.value = "define";
  refTable.value = "";
  refField.value = "";
  if (bizType === "Enum") {
    if (!draft.value.enum) {
      draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
    }
    ensureEnumDefaults();
    draft.value.bizTypeData = undefined;
    draft.value.dataType = DataType.Varchar;
    // 强制 VARCHAR 时清理精度并补默认长度(原类型可能无 length)
    draft.value.precision = undefined;
    draft.value.scale = undefined;
    if (draft.value.length == null) draft.value.length = 32;
  } else if (bizType) {
    draft.value.enum = undefined;
    const def = bizTypes.value.find((b) => b.bizType === bizType);
    if (def) {
      // 首次选 bizType(从无到有)-> 强制用第一个类型;换 bizType -> 按兼容判断
      const isFirstSelection = !prevBizType && bizType;
      let dt = draft.value.dataType;
      if (isFirstSelection || !bizTypeSupports(def, dt)) {
        dt = def.supportedDataTypes[0]?.dataType ?? dt;
        draft.value.dataType = dt;
        // dataType 改变时清理不适用属性(§3.1):如 VARCHAR 遗留 length 切到 TINYINT 必须清除
        cleanupDataType(draft.value, dt);
      }
      applyDefaults(draft.value, def, dt);
      draft.value.bizTypeData = initBizTypeData(def) as Field["bizTypeData"];
    }
  } else {
    draft.value.enum = undefined;
  }
}

// 清理不适用属性(§3.1): VARCHAR 仅 length,DECIMAL 仅 precision/scale,其余无
function cleanupDataType(f: Field, dt: DataType) {
  switch (dt) {
    case DataType.Varchar:
      f.precision = undefined;
      f.scale = undefined;
      break;
    case DataType.Decimal:
      f.length = undefined;
      break;
    default:
      f.length = undefined;
      f.precision = undefined;
      f.scale = undefined;
      break;
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
  cleanupDataType(draft.value, draft.value.dataType);
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
    // 从已有 enum.ref 恢复来源与级联
    if (draft.value?.enum?.ref) {
      enumSource.value = "ref";
      refTable.value = draft.value.enum.ref.code;
      refField.value = draft.value.enum.ref.prop;
    } else {
      enumSource.value = "define";
      refTable.value = "";
      refField.value = "";
      ensureEnumDefaults();
    }
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
  // 内联 enum(定义模式)必填项:名称 / Java 类名 / 枚举值列表非空 / 每项 id 与名称有值
  const e = draft.value.enum;
  if (e && !e.ref) {
    if (!e.name?.trim()) {
      ElMessage.error("名称不能为空");
      return;
    }
    if (!e.className?.trim()) {
      ElMessage.error("Java 类名不能为空");
      return;
    }
    if (e.values.length === 0) {
      ElMessage.error("枚举项不能为空,请至少添加一项");
      return;
    }
    if (e.values.some((v) => !v.id?.trim())) {
      ElMessage.error("每个枚举项的字面量不能为空");
      return;
    }
    if (e.values.some((v) => !v.name?.trim())) {
      ElMessage.error("每个枚举项的名称不能为空");
      return;
    }
    if (e.values.some((v) => v.id && e.values.filter((x) => x.id === v.id).length > 1)) {
      ElMessage.error("枚举项字面量不能重复");
      return;
    }
  }
  // 存值用 code:每个枚举值 code 必填
  if (e && e.hasCode) {
    if (e.values.some((v) => !v.code || !v.code.trim())) {
      ElMessage.error("勾选存值用 code 时,每个枚举项的 code 必填");
      return;
    }
  }
  // 引用模式:必须已选到目标字段
  if (e && e.ref) {
    if (!refTable.value || !refField.value) {
      ElMessage.error("引用枚举需先选择引用表和引用字段");
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
  <el-dialog draggable v-model="visible" title="业务类型" width="720px" top="8vh" :close-on-click-modal="false">
    <div v-if="draft" class="flex flex-col gap-4" style="max-height: 70vh; overflow-y: auto">
      <el-form label-width="90px" class="pr-12" :disabled="store.readOnly">
        <el-form-item label="业务类型" class="mb-0">
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
        <!-- 业务类型描述(无值时保留占位间距,避免下方内容贴住业务类型行) -->
        <div v-if="currentBizType?.description" class="text-12 text-gray-400 ml-90 mb-8">{{ currentBizType.description }}</div>
        <div v-else-if="isEnumBizType || bizTypeDataFields.length" class="mb-16" />

        <!-- bizType=Enum: 枚举特殊配置(内联)-->
        <template v-if="isEnumBizType">
          <template v-if="inlineEnum">
            <!-- 枚举来源 -->
            <el-form-item label="枚举来源" class="mb-16">
              <el-radio-group :model-value="enumSource" @update:model-value="(v: string | number | boolean) => onEnumSourceChange(v as 'define' | 'ref')">
                <el-radio-button value="define">本字段定义</el-radio-button>
                <el-radio-button value="ref">引用已有枚举</el-radio-button>
              </el-radio-group>
            </el-form-item>

            <!-- 模式A: 本字段定义 -->
            <template v-if="enumSource === 'define'">
              <el-form-item label="名称" required class="mb-16">
                <el-input v-model="inlineEnum.name" style="width: 200px" />
              </el-form-item>
              <el-form-item label="Java 类名" required class="mb-16">
                <el-input v-model="inlineEnum.className" style="width: 200px" />
                <span class="text-12 text-gray-400 ml-8 self-center">默认按字段 prop 转换</span>
              </el-form-item>
              <el-form-item label="枚举项" class="mb-0">
                <div class="w-full">
                  <div class="flex items-center mb-8">
                    <el-button size="small" :disabled="store.readOnly" @click="addInlineValue">+ 添加项</el-button>
                    <el-tooltip
                      content="勾选:枚举字面量与数据库保存值不一致,数据库保存 code;不勾:数据库直接保存枚举字面量"
                      placement="top"
                    >
                      <el-checkbox v-model="inlineEnum.hasCode" class="ml-12" @change="onHasCodeChange">存值用 code</el-checkbox>
                    </el-tooltip>
                  </div>
                  <el-table :data="inlineEnum.values" border size="small">
                    <el-table-column label="字面量" min-width="110">
                      <template #default="{ row }"><el-input v-model="row.id" size="small" /></template>
                    </el-table-column>
                    <el-table-column label="名称" min-width="100">
                      <template #default="{ row }"><el-input v-model="row.name" size="small" /></template>
                    </el-table-column>
                    <el-table-column v-if="inlineEnum.hasCode" label="code" min-width="100">
                      <template #default="{ row }">
                        <el-input v-model="row.code" size="small" placeholder="必填" />
                      </template>
                    </el-table-column>
                    <el-table-column label="颜色" min-width="110">
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

            <!-- 模式B: 引用已有(级联: 先表后字段) -->
            <template v-else>
              <el-form-item label="引用表" class="mb-16">
                <el-select v-model="refTable" style="width: 240px" placeholder="选择含枚举的表" @change="onRefTableChange">
                  <el-option v-for="t in enumRefTables" :key="t.code" :label="`${t.name}(${t.code})`" :value="t.code" />
                </el-select>
              </el-form-item>
              <el-form-item label="引用字段" class="mb-16">
                <el-select v-model="refField" style="width: 240px" placeholder="选择定义方枚举字段" :disabled="!refTable" @change="onRefFieldChange">
                  <el-option v-for="f in enumRefFields" :key="f.prop" :label="`${f.name} → ${enumClassName(f)}`" :value="f.prop" />
                </el-select>
              </el-form-item>
              <el-form-item label="引用信息" class="mb-0">
                <div v-if="refTarget" class="w-full">
                  <el-table :data="refTarget.field.enum?.values ?? []" border size="small">
                    <el-table-column prop="id" label="字面量" min-width="100" />
                    <el-table-column prop="name" label="名称" min-width="100" />
                    <el-table-column prop="code" label="Code" min-width="80">
                      <template #default="{ row }">
                        {{ row.code || '-' }}
                      </template>
                    </el-table-column>
                    <el-table-column prop="color" label="颜色" min-width="80">
                      <template #default="{ row }">
                        {{ row.color || '-' }}
                      </template>
                    </el-table-column>
                  </el-table>
                </div>
                <span v-else class="text-12 text-gray-500">(未选择)</span>
              </el-form-item>
            </template>
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
