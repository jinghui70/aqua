<script setup lang="ts">
// 布局分两态:
// - 无项目: 全屏欢迎页(独立,无左树)
// - 有项目: 左分组树 + 多标签工作区 + 状态栏(splitter 可拖动调整树宽度)
// 菜单是原生窗口菜单(Rust 侧)。
import { computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";
import GroupTreeAside from "./GroupTreeAside.vue";
import TabWorkspace from "./TabWorkspace.vue";
import StatusBar from "./StatusBar.vue";
import AppToolbar from "./AppToolbar.vue";
import ExportDialog from "@/components/ExportDialog.vue";
import DataSourceDialog from "@/components/DataSourceDialog.vue";
import DatabaseConfigDialog from "@/components/DatabaseConfigDialog.vue";
import ImportWizard from "@/components/ImportWizard.vue";
import RecentProjectsDialog from "@/components/RecentProjectsDialog.vue";
import ProjectSettingsDialog from "@/components/ProjectSettingsDialog.vue";
import NewProjectDialog from "@/components/NewProjectDialog.vue";

const store = useProjectStore();
const route = useRoute();
const router = useRouter();
// 配置/数据集页覆盖工作区(全屏,不显示分组树+TabWorkspace)
const isFullPage = computed(() => route.path === "/config" || route.path === "/dataset");
// 无项目 -> /welcome;有项目且在 /welcome -> /(工作区)
watch(
  () => store.currentProject,
  (p) => {
    if (!p && route.path !== "/welcome") router.push("/welcome");
    else if (p && route.path === "/welcome") router.push("/");
  },
  { immediate: true }
);
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- 无项目: 全屏欢迎页 -->
    <router-view v-if="!store.currentProject" class="flex-1 overflow-auto" />

    <!-- 有项目: 工具栏 + 工作区布局(splitter 分割) -->
    <template v-else>
      <AppToolbar />
      <router-view v-if="isFullPage" class="flex-1 min-h-0" />
      <el-splitter v-else class="flex-1 min-h-0">
        <el-splitter-panel :size="260" :min="200" :max="500">
          <GroupTreeAside />
        </el-splitter-panel>
        <el-splitter-panel>
          <TabWorkspace />
        </el-splitter-panel>
      </el-splitter>
      <StatusBar />
    </template>

    <!-- 全局弹窗(两态都可用) -->
    <ExportDialog />
    <DataSourceDialog />
    <DatabaseConfigDialog />
    <ImportWizard />
    <RecentProjectsDialog />
    <ProjectSettingsDialog />
    <NewProjectDialog />
  </div>
</template>
