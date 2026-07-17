<script setup lang="ts">
// 导入向导(§6.8):4 步 - 选数据源 -> 选表 -> 选项 -> 确认。
import { computed, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useDataSourceStore } from "@/stores/datasource";
import { useTauri, type TableInfo } from "@/composables/useTauri";
import type { DbConfig, Project } from "@/types/schema";

const ui = useUiStore();
const store = useProjectStore();
const dsStore = useDataSourceStore();
const tauri = useTauri();

const step = ref(0);

// Step1: 数据源
const selectedSource = ref("");
const config = computed<DbConfig | null>(() => {
  const ds = dsStore.sources.find((s) => s.sourceName === selectedSource.value);
  if (!ds) return null;
  return {
    dialect: ds.dialect,
    host: ds.host,
    port: ds.port,
    user: ds.user,
    password: ds.password,
    database: ds.database,
    schema: ds.schema,
  };
});

// Step2: 表
const allTables = ref<TableInfo[]>([]);
const selectedTables = ref<string[]>([]);
const tableFilter = ref("");
const loadingTables = ref(false);
const filteredTables = computed(() =>
  allTables.value.filter((t) =>
    t.name.toLowerCase().includes(tableFilter.value.toLowerCase())
  )
);

// Step3: 选项
const targetGroup = ref("default");
const overwrite = ref(true);
const groups = computed(() => store.currentProject?.groups ?? []);

// 导入结果缓存
const importedProject = ref<Project | null>(null);

function reset() {
  step.value = 0;
  selectedSource.value = "";
  allTables.value = [];
  selectedTables.value = [];
  tableFilter.value = "";
  targetGroup.value = "default";
  overwrite.value = true;
  importedProject.value = null;
}

watch(
  () => ui.importVisible,
  (v) => {
    if (v) reset();
  }
);

async function nextFromSource() {
  if (!config.value) {
    ElMessage.warning("请选择数据源");
    return;
  }
  if (!store.currentProject) {
    ElMessage.warning("请先新建或打开项目");
    return;
  }
  loadingTables.value = true;
  try {
    allTables.value = await tauri.listTables(config.value);
    step.value = 1;
  } catch {
    /* 已提示 */
  } finally {
    loadingTables.value = false;
  }
}

function toggleAll() {
  selectedTables.value =
    selectedTables.value.length === filteredTables.value.length
      ? []
      : filteredTables.value.map((t) => t.name);
}

async function nextFromTables() {
  if (!selectedTables.value.length) {
    ElMessage.warning("请至少选一个表");
    return;
  }
  step.value = 2;
}

async function doImport() {
  if (!config.value || !store.currentProject) return;
  try {
    // 按选中表导入(后端只反解选中表,避免整库 spawn 开销)
    const tableInfos = allTables.value.filter((t) =>
      selectedTables.value.includes(t.name)
    );
    importedProject.value = await tauri.importFromDb(
      config.value,
      tableInfos,
      store.currentProject.basePackage
    );
    const { added, skipped } = store.mergeImportedTables(
      importedProject.value,
      selectedTables.value,
      targetGroup.value,
      overwrite.value
    );
    ElMessage.success(`导入完成: 新增/覆盖 ${added}, 跳过 ${skipped}`);
    ui.importVisible = false;
  } catch {
    /* 已提示 */
  }
}
</script>

<template>
  <el-dialog v-model="ui.importVisible" title="导入向导" width="640px" :close-on-click-modal="false">
    <el-steps :active="step" finish-status="success" simple class="mb-16">
      <el-step title="数据源" />
      <el-step title="选表" />
      <el-step title="选项" />
    </el-steps>

    <!-- Step1 -->
    <div v-if="step === 0" style="min-height: 260px">
      <el-form label-width="80px">
        <el-form-item label="数据源">
          <el-select v-model="selectedSource" placeholder="选择数据源" style="width: 100%">
            <el-option
              v-for="ds in dsStore.sources"
              :key="ds.sourceName"
              :label="`${ds.sourceName} (${ds.dialect})`"
              :value="ds.sourceName"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <el-empty
        v-if="!dsStore.sources.length"
        description="无数据源,请先在「配置>数据源配置」添加"
        :image-size="60"
      />
    </div>

    <!-- Step2 -->
    <div v-else-if="step === 1" style="min-height: 260px">
      <div class="flex items-center gap-8 mb-8">
        <el-input v-model="tableFilter" size="small" placeholder="搜索表" clearable style="width: 200px" />
        <el-button size="small" @click="toggleAll">全选/反选</el-button>
        <span class="text-12 text-gray-400">已选 {{ selectedTables.length }}</span>
      </div>
      <el-checkbox-group v-model="selectedTables" class="flex flex-col gap-4" style="max-height: 220px; overflow-y: auto">
        <el-checkbox v-for="t in filteredTables" :key="t.name" :value="t.name">
          {{ t.name }}<span v-if="t.comment" class="text-12 text-gray-400 ml-8">{{ t.comment }}</span>
        </el-checkbox>
      </el-checkbox-group>
    </div>

    <!-- Step3 -->
    <div v-else-if="step === 2" style="min-height: 260px">
      <el-form label-width="100px">
        <el-form-item label="目标分组">
          <el-select v-model="targetGroup" style="width: 100%">
            <el-option v-for="g in groups" :key="g.code" :label="g.name" :value="g.code" />
          </el-select>
        </el-form-item>
        <el-form-item label="同名表">
          <el-radio-group v-model="overwrite">
            <el-radio :value="true">覆盖</el-radio>
            <el-radio :value="false">跳过</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="确认">
          <span class="text-13 text-gray-500">
            将导入 {{ selectedTables.length }} 个表到分组「{{ targetGroup }}」
          </span>
        </el-form-item>
      </el-form>
    </div>

    <template #footer>
      <el-button v-if="step > 0" @click="step--">上一步</el-button>
      <el-button v-if="step === 0" type="primary" :loading="loadingTables" @click="nextFromSource">
        下一步
      </el-button>
      <el-button v-else-if="step === 1" type="primary" @click="nextFromTables">下一步</el-button>
      <el-button v-else type="primary" @click="doImport">导入</el-button>
    </template>
  </el-dialog>
</template>
