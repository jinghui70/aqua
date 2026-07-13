<script setup lang="ts">
// 业务类型管理(§6.5):左列表 + 右编辑。
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";
import { DataType, type BizTypeDefine } from "@/types/schema";

const store = useProjectStore();
const builtin = useBuiltinStore();
const dataTypes = Object.values(DataType);

const projectBizTypes = computed(() => store.currentProject?.bizTypes ?? []);
// 合并展示:内置(只读)+ 自定义(可改)
const bizTypes = computed<BizTypeDefine[]>(() => [
  ...builtin.bizTypes,
  ...projectBizTypes.value,
]);
const selectedCode = ref("");
const current = computed(() =>
  bizTypes.value.find((b) => b.bizType === selectedCode.value)
);
const isCurrentBuiltin = computed(() =>
  selectedCode.value ? builtin.isBuiltin(selectedCode.value) : false
);

function select(code: string) {
  selectedCode.value = code;
}

async function addBizType() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  try {
    const { value } = await ElMessageBox.prompt("业务类型 code", "新建业务类型", {
      confirmButtonText: "创建",
      cancelButtonText: "取消",
      inputPlaceholder: "Date8",
    });
    if (!value) return;
    const code = value.trim();
    // 重名校验含内置
    if (bizTypes.value.some((b) => b.bizType === code)) {
      ElMessage.error(`${code} 已存在`);
      return;
    }
    const biz: BizTypeDefine = {
      bizType: code,
      name: code,
      supportedDataTypes: [],
    };
    store.currentProject.bizTypes.push(biz);
    selectedCode.value = code;
    ElMessage.success("已创建");
  } catch {
    /* 取消 */
  }
}

// 统计关联该 bizType 的字段(表code.字段code)
function relatedFields(code: string): string[] {
  const res: string[] = [];
  for (const t of store.currentProject?.tables ?? []) {
    for (const f of t.fields) {
      if (f.bizType === code) res.push(`${t.code}.${f.code}`);
    }
  }
  return res;
}

async function removeBizType(code: string) {
  if (builtin.isBuiltin(code)) return; // 内置不可删(UI 已禁,防御)
  const related = relatedFields(code);
  const msg = related.length
    ? [
        `业务类型 <b>${code}</b> 已被 <b>${related.length}</b> 个字段关联:`,
        related.slice(0, 8).join("、") + (related.length > 8 ? " …" : ""),
        "删除将同时清除这些字段的业务类型设置,请慎重。确认删除?",
      ].join("<br/>")
    : `确认删除业务类型 ${code}?`;
  try {
    await ElMessageBox.confirm(msg, "删除业务类型", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      dangerouslyUseHTMLString: true,
    });
    // 级联清除关联字段的 bizType / bizTypeData
    for (const t of store.currentProject!.tables) {
      for (const f of t.fields) {
        if (f.bizType === code) {
          f.bizType = undefined;
          f.bizTypeData = undefined;
        }
      }
    }
    const arr = store.currentProject!.bizTypes;
    const idx = arr.findIndex((b) => b.bizType === code);
    if (idx >= 0) arr.splice(idx, 1);
    if (selectedCode.value === code) selectedCode.value = "";
    ElMessage.success(
      related.length ? `已删除,并清除 ${related.length} 个字段的关联` : "已删除"
    );
  } catch {
    /* 取消 */
  }
}

// supportedDataTypes 子表
function addSupported() {
  current.value?.supportedDataTypes.push({ dataType: DataType.Varchar });
}
function removeSupported(idx: number) {
  current.value?.supportedDataTypes.splice(idx, 1);
}

// bizTypeData.fields 子表
function ensureBizTypeData() {
  if (current.value && !current.value.bizTypeData) {
    current.value.bizTypeData = { fields: [] };
  }
}
function addDataField() {
  ensureBizTypeData();
  current.value!.bizTypeData!.fields.push({ name: "", type: "string" });
}
function removeDataField(idx: number) {
  current.value?.bizTypeData?.fields.splice(idx, 1);
}
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex">
    <!-- 左列表 -->
    <div class="w-220 border-r border-gray-200 flex flex-col flex-shrink-0">
      <div
        class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14"
      >
        <span>业务类型</span>
        <el-button size="small" type="primary" link @click="addBizType">+ 新建</el-button>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div
          v-for="b in bizTypes"
          :key="b.bizType"
          class="flex items-center justify-between px-12 py-8 cursor-pointer text-13 hover:bg-gray-100"
          :class="{ 'bg-blue-50': b.bizType === selectedCode }"
          @click="select(b.bizType)"
        >
          <span class="flex items-center gap-6">
            <el-tag v-if="builtin.isBuiltin(b.bizType)" size="small" type="info" effect="plain">内置</el-tag>
            {{ b.name }} ({{ b.bizType }})
          </span>
          <el-button
            v-if="!builtin.isBuiltin(b.bizType)"
            size="small"
            link
            type="danger"
            @click.stop="removeBizType(b.bizType)"
          >删</el-button>
        </div>
        <el-empty v-if="!bizTypes.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右编辑 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <el-alert
          v-if="isCurrentBuiltin"
          title="内置业务类型,只读(不可删改)"
          type="info"
          :closable="false"
          class="mb-12"
        />
        <el-form label-width="130px" class="max-w-3xl">
          <el-form-item label="bizType (code)">
            <el-input :model-value="current.bizType" disabled />
          </el-form-item>
          <el-form-item label="名称">
            <el-input v-model="current.name" :disabled="isCurrentBuiltin" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input v-model="current.description" :disabled="isCurrentBuiltin" />
          </el-form-item>
        </el-form>

        <!-- supportedDataTypes -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          支持的数据类型
          <el-button v-if="!isCurrentBuiltin" size="small" type="primary" link @click="addSupported">+ 添加</el-button>
        </div>
        <el-table :data="current.supportedDataTypes" border size="small">
          <el-table-column label="逻辑类型" width="140">
            <template #default="{ row }">
              <el-select v-model="row.dataType" size="small" :disabled="isCurrentBuiltin">
                <el-option v-for="dt in dataTypes" :key="dt" :label="dt" :value="dt" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="默认长度" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultLength" size="small" :controls="false" :min="1" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column label="默认精度" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultPrecision" size="small" :controls="false" :min="1" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column label="默认小数位" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultScale" size="small" :controls="false" :min="0" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column v-if="!isCurrentBuiltin" label="操作" width="70" align="center">
            <template #default="{ $index }">
              <el-button size="small" link type="danger" @click="removeSupported($index)">删</el-button>
            </template>
          </el-table-column>
        </el-table>

        <!-- bizTypeData.fields -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          参数配置 (bizTypeData.fields)
          <el-button v-if="!isCurrentBuiltin" size="small" type="primary" link @click="addDataField">+ 添加</el-button>
        </div>
        <el-table :data="current.bizTypeData?.fields ?? []" border size="small">
          <el-table-column label="参数名" width="150">
            <template #default="{ row }">
              <el-input v-model="row.name" size="small" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column label="类型" width="120">
            <template #default="{ row }">
              <el-select v-model="row.type" size="small" :disabled="isCurrentBuiltin">
                <el-option label="string" value="string" />
                <el-option label="number" value="number" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="默认值" width="140">
            <template #default="{ row }">
              <el-input-number
                v-if="row.type === 'number'"
                v-model="row.default"
                size="small"
                :controls="false"
                :disabled="isCurrentBuiltin"
              />
              <el-input
                v-else
                v-model="row.default"
                size="small"
                :disabled="isCurrentBuiltin"
              />
            </template>
          </el-table-column>
          <el-table-column label="描述" min-width="150">
            <template #default="{ row }">
              <el-input v-model="row.description" size="small" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column label="必填" width="60" align="center">
            <template #default="{ row }">
              <el-checkbox v-model="row.required" :disabled="isCurrentBuiltin" />
            </template>
          </el-table-column>
          <el-table-column v-if="!isCurrentBuiltin" label="操作" width="70" align="center">
            <template #default="{ $index }">
              <el-button size="small" link type="danger" @click="removeDataField($index)">删</el-button>
            </template>
          </el-table-column>
        </el-table>
      </template>
      <el-empty v-else description="选择或新建业务类型" />
    </div>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
