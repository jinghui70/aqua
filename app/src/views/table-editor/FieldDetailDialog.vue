<script setup lang="ts">
// 字段完整编辑弹窗:补齐行内表格放不下的属性(autoGenerate.param / bizTypeData / enum)。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { DataType, type Field, type InlineEnum } from "@/types/schema";
import { useProjectStore } from "@/stores/project";

const props = defineProps<{ modelValue: boolean; field: Field | null }>();
const emit = defineEmits<{ "update:modelValue": [boolean] }>();

const store = useProjectStore();
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
const bizTypes = computed(() => store.currentProject?.bizTypes ?? []);
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
function setBizTypeDataValue(fieldName: string, value: unknown) {
  if (!draft.value) return;
  if (bizTypeDataFields.value.length === 1) {
    draft.value.bizTypeData = value;
  } else {
    const obj = (draft.value.bizTypeData as Record<string, unknown>) ?? {};
    obj[fieldName] = value;
    draft.value.bizTypeData = obj;
  }
}

// ===== enum(无/引用/内联)=====
type EnumMode = "none" | "ref" | "inline";
const enumMode = ref<EnumMode>("none");
const globalEnums = computed(() => store.currentProject?.enums ?? []);
const isVarchar = computed(() => draft.value?.dataType === DataType.Varchar);

function syncEnumMode() {
  const e = draft.value?.enum;
  if (!e) enumMode.value = "none";
  else if (typeof e === "string") enumMode.value = "ref";
  else enumMode.value = "inline";
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
  else if (mode === "ref") draft.value.enum = "";
  else
    draft.value.enum = { name: "", hasCode: false, values: [] } as InlineEnum;
}

const inlineEnum = computed(() =>
  draft.value && typeof draft.value.enum === "object"
    ? (draft.value.enum as InlineEnum)
    : null
);

// 引用全局枚举的可写绑定(v-model 不能绑类型断言表达式)
const refEnumCode = computed<string>({
  get: () => (typeof draft.value?.enum === "string" ? draft.value.enum : ""),
  set: (v) => {
    if (draft.value) draft.value.enum = v;
  },
});

function addInlineValue() {
  inlineEnum.value?.values.push({ id: "", name: "" });
}
function removeInlineValue(idx: number) {
  inlineEnum.value?.values.splice(idx, 1);
}

// ===== 保存 =====
function save() {
  if (!draft.value || !props.field) return;
  // enum 只支持 VARCHAR
  if (draft.value.enum && draft.value.dataType !== DataType.Varchar) {
    ElMessage.error("enum 只支持 VARCHAR 类型");
    return;
  }
  // 写回原字段(保持引用,Object.assign)
  Object.keys(props.field).forEach((k) => delete (props.field as any)[k]);
  Object.assign(props.field, draft.value);
  visible.value = false;
  ElMessage.success("已保存");
}
</script>

<template>
  <el-dialog v-model="visible" title="字段编辑" width="600px" top="6vh">
    <div v-if="draft" class="flex flex-col gap-4" style="max-height: 70vh; overflow-y: auto">
      <!-- 基本 -->
      <el-form label-width="90px" class="pr-12">
        <el-form-item label="code">
          <el-input
            v-model="draft.code"
            @input="draft.code = draft.code.toUpperCase()"
          />
        </el-form-item>
        <el-form-item label="prop">
          <el-input v-model="draft.prop" />
        </el-form-item>
        <el-form-item label="名称">
          <el-input v-model="draft.name" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="draft.dataType" style="width: 160px">
            <el-option v-for="dt in dataTypes" :key="dt" :label="dt" :value="dt" />
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
        <el-form-item label="默认值">
          <el-input v-model="draft.defaultValue" placeholder="DDL DEFAULT 子句" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="draft.comment" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>

      <!-- autoGenerate -->
      <el-divider content-position="left">自动生成</el-divider>
      <el-form label-width="90px" class="pr-12">
        <el-form-item label="启用">
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

      <!-- 业务类型 -->
      <el-divider content-position="left">业务类型</el-divider>
      <el-form label-width="90px" class="pr-12">
        <el-form-item label="bizType">
          <el-select v-model="draft.bizType" clearable placeholder="-" style="width: 200px">
            <el-option v-for="b in bizTypes" :key="b.bizType" :label="b.name" :value="b.bizType" />
          </el-select>
        </el-form-item>
        <!-- bizTypeData 动态表单 -->
        <el-form-item
          v-for="bf in bizTypeDataFields"
          :key="bf.name"
          :label="bf.name"
        >
          <el-input
            v-if="bf.type === 'string'"
            :model-value="getBizTypeDataValue(bf.name) as string"
            :placeholder="bf.description"
            @update:model-value="(v: string) => setBizTypeDataValue(bf.name, v)"
          />
          <el-input-number
            v-else
            :model-value="getBizTypeDataValue(bf.name) as number"
            :controls="false"
            @update:model-value="(v: number | undefined) => setBizTypeDataValue(bf.name, v)"
          />
        </el-form-item>
      </el-form>

      <!-- enum(仅 VARCHAR)-->
      <el-divider content-position="left">枚举</el-divider>
      <el-form label-width="90px" class="pr-12">
        <el-form-item label="枚举类型">
          <el-radio-group
            v-model="enumMode"
            :disabled="!isVarchar"
            @change="(m: any) => onEnumModeChange(m)"
          >
            <el-radio value="none">无</el-radio>
            <el-radio value="ref">引用全局</el-radio>
            <el-radio value="inline">内联</el-radio>
          </el-radio-group>
          <span v-if="!isVarchar" class="text-12 text-gray-400 ml-8">仅 VARCHAR 支持</span>
        </el-form-item>
        <!-- 引用全局 -->
        <el-form-item v-if="enumMode === 'ref'" label="选择枚举">
          <el-select v-model="refEnumCode" placeholder="选全局枚举" style="width: 220px">
            <el-option v-for="e in globalEnums" :key="e.code" :label="`${e.name} (${e.code})`" :value="e.code" />
          </el-select>
        </el-form-item>
        <!-- 内联 -->
        <template v-if="enumMode === 'inline' && inlineEnum">
          <el-form-item label="枚举名">
            <el-input v-model="inlineEnum.name" style="width: 200px" />
            <el-checkbox v-model="inlineEnum.hasCode" class="ml-12">hasCode</el-checkbox>
          </el-form-item>
          <el-form-item label="枚举值">
            <div class="w-full">
              <el-button size="small" @click="addInlineValue" class="mb-8">+ 添加值</el-button>
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
                    <el-button size="small" link type="danger" @click="removeInlineValue($index)">删</el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </el-form-item>
        </template>
      </el-form>
    </div>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>
