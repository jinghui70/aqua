<script setup lang="ts">
// 数据库配置弹窗:管理 JDBC 驱动安装 + 数据库显隐。
import { ref, onMounted } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useTauri } from "@/composables/useTauri";
import { useDatabaseStore } from "@/stores/database";
import { pickDriverJar } from "@/composables/useFileDialog";
import type { DatabaseInfo } from "@/types/schema";

const ui = useUiStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

const loading = ref(false);

async function refresh() {
  loading.value = true;
  try {
    await dbStore.load();
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  if (!dbStore.loaded) void refresh();
});

/** 是否可安装驱动(reverseSupported && !builtinDriver,本任务即 Oracle)。 */
function canInstall(d: DatabaseInfo): boolean {
  return d.reverseSupported && !d.builtinDriver;
}

/** 驱动状态文案。 */
function driverText(d: DatabaseInfo): string {
  if (d.builtinDriver) return "内置";
  if (d.installed) return d.installedJar ?? "已装";
  return "未装";
}

async function onInstall(d: DatabaseInfo) {
  const jar = await pickDriverJar();
  if (!jar) return;
  try {
    await tauri.installDriver(d.name, jar);
    ElMessage.success(`已安装 ${d.label} 驱动`);
    await refresh();
  } catch {
    /* useTauri 已提示 */
  }
}

async function onUninstall(d: DatabaseInfo) {
  try {
    await tauri.uninstallDriver(d.name);
    ElMessage.success(`已卸载 ${d.label} 驱动`);
    await refresh();
  } catch {
    /* useTauri 已提示 */
  }
}

async function onToggleVisible(d: DatabaseInfo, visible: boolean | string | number) {
  // visible = !hidden
  try {
    await tauri.setDatabaseHidden(d.name, !visible);
    await refresh();
  } catch {
    /* useTauri 已提示 */
  }
}
</script>

<template>
  <el-dialog v-model="ui.databaseConfigVisible" title="数据库配置" width="680px">
    <el-table v-loading="loading" :data="dbStore.databases" size="small">
      <el-table-column label="数据库" prop="label" min-width="120" />
      <el-table-column label="类型" width="80">
        <template #default="{ row }">
          {{ row.category === "native" ? "native" : "JDBC" }}
        </template>
      </el-table-column>
      <el-table-column label="驱动" min-width="120">
        <template #default="{ row }">
          <span :class="row.installed || row.builtinDriver ? 'text-gray-600' : 'text-gray-400'">
            {{ driverText(row) }}
          </span>
        </template>
      </el-table-column>
      <el-table-column label="显示" width="80" align="center">
        <template #default="{ row }">
          <el-switch
            :model-value="!row.hidden"
            @change="(v: boolean | string | number) => onToggleVisible(row, v)"
          />
        </template>
      </el-table-column>
      <el-table-column label="操作" width="100" align="center">
        <template #default="{ row }">
          <el-button
            v-if="canInstall(row) && !row.installed"
            size="small"
            type="primary"
            link
            @click="onInstall(row)"
          >
            安装
          </el-button>
          <el-button
            v-else-if="row.installed"
            size="small"
            type="danger"
            link
            @click="onUninstall(row)"
          >
            卸载
          </el-button>
          <span v-else class="text-gray-300 text-12">—</span>
        </template>
      </el-table-column>
    </el-table>
    <div class="mt-12 text-12 text-gray-400">
      native(MySQL/PostgreSQL)与 H2 驱动内置;Oracle 等需安装 JDBC 驱动 jar。隐藏的数据库不出现在生成/反解下拉。
    </div>
  </el-dialog>
</template>
