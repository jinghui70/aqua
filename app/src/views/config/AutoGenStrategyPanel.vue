<script setup lang="ts">
// 自动生成策略管理:左列表(内置+自定义)+ 右只读展示 + 新建/编辑共用弹窗。
import { computed, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";
import { collectRelatedTables, buildCascadePrompt } from "@/utils/cascade";
import type { AutoGenStrategyDefine } from "@/types/schema";

const store = useProjectStore();
const builtin = useBuiltinStore();

const projectStrategies = computed(() => store.currentProject?.autoGenStrategies ?? []);
const strategies = computed<AutoGenStrategyDefine[]>(() => [
  ...builtin.autoGenStrategies,
  ...projectStrategies.value,
]);
const selectedCode = ref("");
const current = computed(() => strategies.value.find((s) => s.code === selectedCode.value));
const isCurrentBuiltin = computed(() =>
  selectedCode.value ? builtin.isBuiltinStrategy(selectedCode.value) : false
);
const canEdit = computed(() => !isCurrentBuiltin.value && !store.readOnly);

function select(code: string) {
  selectedCode.value = code;
}

// ===== 新建/编辑共用弹窗 =====
const dialogVisible = ref(false);
const dialogMode = ref<"new" | "edit">("new");
const dialogCode = ref("");
const dialogName = ref("");
const dialogHasParam = ref(false);
const dialogParamDesc = ref("");

function openNew() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  dialogMode.value = "new";
  dialogCode.value = "";
  dialogName.value = "";
  dialogHasParam.value = false;
  dialogParamDesc.value = "";
  dialogVisible.value = true;
}

function openEdit() {
  if (!current.value) return;
  dialogMode.value = "edit";
  dialogCode.value = current.value.code;
  dialogName.value = current.value.name;
  dialogHasParam.value = current.value.paramDesc != null;
  dialogParamDesc.value = current.value.paramDesc ?? "";
  dialogVisible.value = true;
}

function confirmDialog() {
  const code = dialogCode.value.trim();
  if (!code) {
    ElMessage.warning("code 不能为空");
    return;
  }
  const paramDesc = dialogHasParam.value && dialogParamDesc.value.trim()
    ? dialogParamDesc.value.trim()
    : undefined;

  if (dialogMode.value === "new") {
    if (strategies.value.some((s) => s.code === code)) {
      ElMessage.error(`${code} 已存在`);
      return;
    }
    const strategy: AutoGenStrategyDefine = {
      code,
      name: dialogName.value.trim() || code,
      paramDesc,
    };
    store.currentProject!.autoGenStrategies = [...projectStrategies.value, strategy];
    selectedCode.value = code;
    ElMessage.success("已创建");
  } else {
    // 编辑:code 不变,更新 name/paramDesc
    const s = projectStrategies.value.find((x) => x.code === code);
    if (s) {
      s.name = dialogName.value.trim() || code;
      s.paramDesc = paramDesc;
    }
    ElMessage.success("已保存");
  }
  dialogVisible.value = false;
}

async function removeStrategy(code: string) {
  if (builtin.isBuiltinStrategy(code)) return;
  const related = collectRelatedTables(store.currentProject, (f) => f.autoGenerate?.strategy === code);
  const msg = buildCascadePrompt("自动生成策略", code, related);
  try {
    await ElMessageBox.confirm(msg, "删除策略", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      dangerouslyUseHTMLString: true,
    });
    for (const t of store.currentProject!.tables) {
      for (const f of t.fields) {
        if (f.autoGenerate?.strategy === code) {
          f.autoGenerate = undefined;
        }
      }
    }
    store.currentProject!.autoGenStrategies = projectStrategies.value.filter((s) => s.code !== code);
    if (selectedCode.value === code) selectedCode.value = "";
    ElMessage.success(related.length ? `已删除,并清除 ${related.length} 张表的关联` : "已删除");
  } catch {
    /* 取消 */
  }
}
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex">
    <!-- 左列表 -->
    <div class="w-220 border-r border-gray-200 flex flex-col flex-shrink-0">
      <div class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14">
        <span>自动生成策略</span>
        <el-button v-if="!store.readOnly" size="small" type="primary" link @click="openNew">+ 新建</el-button>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div
          v-for="s in strategies"
          :key="s.code"
          class="flex items-center justify-between px-12 py-8 cursor-pointer text-13 hover:bg-gray-100"
          :class="{ 'bg-blue-50': s.code === selectedCode }"
          @click="select(s.code)"
        >
          <span class="flex items-center gap-6">
            {{ s.code }} ({{ s.name }})
            <el-tag v-if="builtin.isBuiltinStrategy(s.code)" size="small" type="info" effect="plain">内置</el-tag>
          </span>
        </div>
        <el-empty v-if="!strategies.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右只读展示 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <div class="flex items-center mb-16">
          <div class="text-20 font-bold mr-20">{{ current.code }}</div>
          <el-button v-if="canEdit" size="small" type="primary" link @click="openEdit">修改</el-button>
          <el-button v-if="canEdit" size="small" link type="danger" @click="removeStrategy(current.code)">删除</el-button>
        </div>
        <div class="flex flex-col gap-8 text-14">
          <div><span class="text-gray-500 inline-block w-80">策略代码:</span>{{ current.code }}</div>
          <div><span class="text-gray-500 inline-block w-80">策略名称:</span>{{ current.name }}</div>
          <div v-if="current.paramDesc"><span class="text-gray-500 inline-block w-80">参数说明:</span>{{ current.paramDesc }}</div>
        </div>
      </template>
      <el-empty v-else description="选择或新建策略" />
    </div>

    <!-- 新建/编辑共用弹窗 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogMode === 'new' ? '新建策略' : '修改策略'"
      width="420px"
      :close-on-click-modal="false"
    >
      <el-form label-width="80px">
        <el-form-item label="code">
          <el-input v-model="dialogCode" :readonly="dialogMode === 'edit'" placeholder="如 seq" />
        </el-form-item>
        <el-form-item label="名称">
          <el-input v-model="dialogName" placeholder="如 序列号" />
        </el-form-item>
        <el-form-item label="有参数">
          <el-switch v-model="dialogHasParam" />
        </el-form-item>
        <el-form-item v-if="dialogHasParam" label="参数说明">
          <el-input v-model="dialogParamDesc" placeholder="如:序列名" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmDialog">{{ dialogMode === "new" ? "创建" : "保存" }}</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
