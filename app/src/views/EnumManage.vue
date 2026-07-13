<script setup lang="ts">
// 枚举管理(§6.6):全局枚举列表 + 编辑。
import { computed, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";
import { collectRelatedTables, buildCascadePrompt } from "@/utils/cascade";
import type { EnumDefine } from "@/types/schema";

const store = useProjectStore();
const tauri = useTauri();

// §3.5 预置 13 色
const COLORS = [
  "success", "error", "warning", "info", "primary", "danger",
  "red", "orange", "yellow", "green", "blue", "purple", "grey",
];

const enums = computed(() => store.currentProject?.enums ?? []);
const selectedCode = ref("");
const current = computed(() =>
  enums.value.find((e) => e.code === selectedCode.value)
);

function select(code: string) {
  selectedCode.value = code;
}

async function addEnum() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  try {
    const { value } = await ElMessageBox.prompt("枚举 code", "新建枚举", {
      confirmButtonText: "创建",
      cancelButtonText: "取消",
      inputPlaceholder: "EnumGender",
    });
    if (!value) return;
    const code = value.trim();
    if (enums.value.some((e) => e.code === code)) {
      ElMessage.error(`${code} 已存在`);
      return;
    }
    const def: EnumDefine = {
      code,
      name: code,
      package: "enum",
      hasCode: false,
      values: [],
    };
    store.currentProject.enums.push(def);
    selectedCode.value = code;
    ElMessage.success("已创建");
  } catch {
    /* 取消 */
  }
}

async function removeEnum(code: string) {
  // 引用该全局枚举的字段(field.enum 为 string 引用;内联枚举为对象,不受影响)
  const related = collectRelatedTables(
    store.currentProject,
    (f) => typeof f.enum === "string" && f.enum === code
  );
  const msg = buildCascadePrompt("枚举", code, related);
  try {
    await ElMessageBox.confirm(msg, "删除枚举", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      dangerouslyUseHTMLString: true,
    });
    // 级联清除:引用该 code 的字段清 enum;若 bizType=Enum 一并清 bizType(避免无 enum 的不一致)
    for (const t of store.currentProject!.tables) {
      for (const f of t.fields) {
        if (typeof f.enum === "string" && f.enum === code) {
          f.enum = undefined;
          if (f.bizType === "Enum") f.bizType = undefined;
        }
      }
    }
    const arr = store.currentProject!.enums;
    const idx = arr.findIndex((e) => e.code === code);
    if (idx >= 0) arr.splice(idx, 1);
    if (selectedCode.value === code) selectedCode.value = "";
    ElMessage.success(
      related.length ? `已删除,并清除 ${related.length} 张表的关联` : "已删除"
    );
  } catch {
    /* 取消 */
  }
}

function addValue() {
  current.value?.values.push({ id: "", name: "" });
}
function removeValue(idx: number) {
  current.value?.values.splice(idx, 1);
}

// Java 预览:选中枚举 + 编辑内容变化实时刷新
const javaPreview = ref("");
async function refreshJava() {
  if (!store.currentProject || !current.value) {
    javaPreview.value = "";
    return;
  }
  try {
    javaPreview.value = await tauri.generateEnum(
      store.currentProject,
      current.value.code
    );
  } catch {
    /* 已提示 */
  }
}
watch(current, refreshJava, { immediate: true });
// 深度监听选中枚举的编辑(values/hasCode/package),实时刷新预览
watch(
  () => (current.value ? JSON.stringify(current.value) : ""),
  refreshJava
);

async function copyJava() {
  await navigator.clipboard.writeText(javaPreview.value);
  ElMessage.success("已复制");
}
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex">
    <!-- 左列表 -->
    <div class="w-220 border-r border-gray-200 flex flex-col flex-shrink-0">
      <div
        class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14"
      >
        <span>枚举</span>
        <el-button size="small" type="primary" link @click="addEnum">+ 新建</el-button>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div
          v-for="e in enums"
          :key="e.code"
          class="flex items-center justify-between px-12 py-8 cursor-pointer text-13 hover:bg-gray-100"
          :class="{ 'bg-blue-50': e.code === selectedCode }"
          @click="select(e.code)"
        >
          <span>{{ e.name }} ({{ e.code }})</span>
          <el-button size="small" link type="danger" @click.stop="removeEnum(e.code)">删</el-button>
        </div>
        <el-empty v-if="!enums.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右编辑 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <el-form label-width="100px" class="max-w-3xl">
          <el-form-item label="code">
            <el-input :model-value="current.code" disabled />
          </el-form-item>
          <el-form-item label="名称">
            <el-input v-model="current.name" />
          </el-form-item>
          <el-form-item label="package">
            <el-input v-model="current.package" />
          </el-form-item>
          <el-form-item label="hasCode">
            <el-switch v-model="current.hasCode" />
            <span class="ml-8 text-12 text-gray-400">
              开启后每个值必须有 code(数据库存 code)
            </span>
          </el-form-item>
        </el-form>

        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          枚举值
          <el-button size="small" type="primary" link @click="addValue">+ 添加</el-button>
        </div>
        <el-table :data="current.values" border size="small">
          <el-table-column label="id" width="150">
            <template #default="{ row }">
              <el-input v-model="row.id" size="small" placeholder="MALE" />
            </template>
          </el-table-column>
          <el-table-column label="名称" width="120">
            <template #default="{ row }">
              <el-input v-model="row.name" size="small" placeholder="男" />
            </template>
          </el-table-column>
          <el-table-column label="code" width="120">
            <template #default="{ row }">
              <el-input
                v-model="row.code"
                size="small"
                :placeholder="current.hasCode ? '必填' : '可选'"
              />
            </template>
          </el-table-column>
          <el-table-column label="颜色" width="140">
            <template #default="{ row }">
              <el-select v-model="row.color" size="small" clearable placeholder="-">
                <el-option v-for="c in COLORS" :key="c" :label="c" :value="c" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="70" align="center">
            <template #default="{ $index }">
              <el-button size="small" link type="danger" @click="removeValue($index)">删</el-button>
            </template>
          </el-table-column>
        </el-table>

        <!-- Java 预览 -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          Java 枚举类
          <el-button size="small" link @click="copyJava">复制</el-button>
        </div>
        <el-input
          :model-value="javaPreview"
          type="textarea"
          :rows="14"
          readonly
          class="font-mono"
        />
      </template>
      <el-empty v-else description="选择或新建枚举" />
    </div>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
