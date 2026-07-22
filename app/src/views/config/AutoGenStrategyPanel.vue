<script setup lang="ts">
// 自动生成策略管理:左列表(内置+自定义)+ 右编辑。常驻配置中心面板。
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
const isReadonly = computed(() => isCurrentBuiltin.value || store.readOnly);

function select(code: string) {
  selectedCode.value = code;
}

// 新建弹窗
const newVisible = ref(false);
const newCode = ref("");
const newName = ref("");
const newHasParam = ref(false);
const newParamDesc = ref("");

function addStrategy() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  newCode.value = "";
  newName.value = "";
  newHasParam.value = false;
  newParamDesc.value = "";
  newVisible.value = true;
}

function confirmNew() {
  const code = newCode.value.trim();
  if (!code) {
    ElMessage.warning("code 不能为空");
    return;
  }
  if (strategies.value.some((s) => s.code === code)) {
    ElMessage.error(`${code} 已存在`);
    return;
  }
  const strategy: AutoGenStrategyDefine = {
    code,
    name: newName.value.trim() || code,
    paramDesc: newHasParam.value && newParamDesc.value.trim() ? newParamDesc.value.trim() : undefined,
  };
  store.currentProject!.autoGenStrategies = [...projectStrategies.value, strategy];
  selectedCode.value = code;
  newVisible.value = false;
  ElMessage.success("已创建");
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
    // 级联清除引用字段的 autoGenerate
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
        <el-button v-if="!store.readOnly" size="small" type="primary" link @click="addStrategy">+ 新建</el-button>
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
          <el-button
            v-if="!builtin.isBuiltinStrategy(s.code) && !store.readOnly"
            size="small"
            link
            type="danger"
            @click.stop="removeStrategy(s.code)"
          >删</el-button>
        </div>
        <el-empty v-if="!strategies.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右编辑 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <el-alert
          v-if="isCurrentBuiltin"
          title="内置策略,只读"
          type="info"
          :closable="false"
          class="mb-12"
        />
        <div class="text-20 font-bold mb-12">{{ current.code }}</div>
        <div class="flex gap-12 mb-12">
          <el-input v-model="current.name" placeholder="名称" style="width: 200px" :readonly="isReadonly" />
        </div>
        <el-form label-width="100px" class="max-w-480">
          <el-form-item label="有参数">
            <el-switch
              :model-value="current.paramDesc != null"
              :disabled="isReadonly"
              @change="(v: any) => { current!.paramDesc = v ? '' : undefined }"
            />
          </el-form-item>
          <el-form-item v-if="current.paramDesc != null" label="参数说明">
            <el-input v-model="current.paramDesc" placeholder="如:yyyy-MM-dd HH:mm:ss" :readonly="isReadonly" />
          </el-form-item>
        </el-form>
      </template>
      <el-empty v-else description="选择或新建策略" />
    </div>

    <!-- 新建弹窗 -->
    <el-dialog v-model="newVisible" title="新建策略" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="code">
          <el-input v-model="newCode" placeholder="如 seq" />
        </el-form-item>
        <el-form-item label="名称">
          <el-input v-model="newName" placeholder="如 序列号" />
        </el-form-item>
        <el-form-item label="有参数">
          <el-switch v-model="newHasParam" />
        </el-form-item>
        <el-form-item v-if="newHasParam" label="参数说明">
          <el-input v-model="newParamDesc" placeholder="如:序列名" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="newVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmNew">创建</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
