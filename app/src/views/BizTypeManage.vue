<script setup lang="ts">
// 业务类型管理(§6.5):左列表 + 右编辑。
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { DataType, type BizTypeDefine } from "@/types/schema";

const store = useProjectStore();
const dataTypes = Object.values(DataType);

const bizTypes = computed(() => store.currentProject?.bizTypes ?? []);
const selectedCode = ref("");
const current = computed(() =>
  bizTypes.value.find((b) => b.bizType === selectedCode.value)
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

async function removeBizType(code: string) {
  try {
    await ElMessageBox.confirm(`确认删除业务类型 ${code}?`, "删除", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
    const arr = store.currentProject!.bizTypes;
    const idx = arr.findIndex((b) => b.bizType === code);
    if (idx >= 0) arr.splice(idx, 1);
    if (selectedCode.value === code) selectedCode.value = "";
    ElMessage.success("已删除");
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
          <span>{{ b.name }} ({{ b.bizType }})</span>
          <el-button size="small" link type="danger" @click.stop="removeBizType(b.bizType)">删</el-button>
        </div>
        <el-empty v-if="!bizTypes.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右编辑 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <el-form label-width="130px" class="max-w-3xl">
          <el-form-item label="bizType (code)">
            <el-input :model-value="current.bizType" disabled />
          </el-form-item>
          <el-form-item label="名称">
            <el-input v-model="current.name" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input v-model="current.description" />
          </el-form-item>
        </el-form>

        <!-- supportedDataTypes -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          支持的数据类型
          <el-button size="small" type="primary" link @click="addSupported">+ 添加</el-button>
        </div>
        <el-table :data="current.supportedDataTypes" border size="small">
          <el-table-column label="逻辑类型" width="140">
            <template #default="{ row }">
              <el-select v-model="row.dataType" size="small">
                <el-option v-for="dt in dataTypes" :key="dt" :label="dt" :value="dt" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="默认长度" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultLength" size="small" :controls="false" :min="1" />
            </template>
          </el-table-column>
          <el-table-column label="默认精度" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultPrecision" size="small" :controls="false" :min="1" />
            </template>
          </el-table-column>
          <el-table-column label="默认小数位" width="120">
            <template #default="{ row }">
              <el-input-number v-model="row.defaultScale" size="small" :controls="false" :min="0" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="70" align="center">
            <template #default="{ $index }">
              <el-button size="small" link type="danger" @click="removeSupported($index)">删</el-button>
            </template>
          </el-table-column>
        </el-table>

        <!-- bizTypeData.fields -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          参数配置 (bizTypeData.fields)
          <el-button size="small" type="primary" link @click="addDataField">+ 添加</el-button>
        </div>
        <el-table :data="current.bizTypeData?.fields ?? []" border size="small">
          <el-table-column label="参数名" width="150">
            <template #default="{ row }">
              <el-input v-model="row.name" size="small" />
            </template>
          </el-table-column>
          <el-table-column label="类型" width="120">
            <template #default="{ row }">
              <el-select v-model="row.type" size="small">
                <el-option label="string" value="string" />
                <el-option label="number" value="number" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="描述" min-width="150">
            <template #default="{ row }">
              <el-input v-model="row.description" size="small" />
            </template>
          </el-table-column>
          <el-table-column label="必填" width="60" align="center">
            <template #default="{ row }">
              <el-checkbox v-model="row.required" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="70" align="center">
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
