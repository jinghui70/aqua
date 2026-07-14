<script setup lang="ts">
// 布局分两态:
// - 无项目: 全屏欢迎页(独立,无左树)
// - 有项目: 左分组树 + 多标签工作区 + 状态栏(splitter 可拖动调整树宽度)
// 菜单是原生窗口菜单(Rust 侧)。
import { useProjectStore } from "@/stores/project";
import Welcome from "@/views/Welcome.vue";
import GroupTreeAside from "./GroupTreeAside.vue";
import TabWorkspace from "./TabWorkspace.vue";
import StatusBar from "./StatusBar.vue";
import ExportDialog from "@/components/ExportDialog.vue";
import DataSourceDialog from "@/components/DataSourceDialog.vue";
import ImportWizard from "@/components/ImportWizard.vue";
import RecentProjectsDialog from "@/components/RecentProjectsDialog.vue";
import ProjectSettingsDialog from "@/components/ProjectSettingsDialog.vue";
import NewProjectDialog from "@/components/NewProjectDialog.vue";

const store = useProjectStore();
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- 无项目: 全屏欢迎页 -->
    <Welcome v-if="!store.currentProject" class="flex-1 overflow-auto" />

    <!-- 有项目: 工作区布局(splitter 分割) -->
    <template v-else>
      <el-splitter class="flex-1" style="height: 0">
        <template #pane-1>
          <GroupTreeAside />
        </template>
        <template #pane-2>
          <TabWorkspace />
        </template>
      </el-splitter>
      <StatusBar />
    </template>

    <!-- 全局弹窗(两态都可用) -->
    <ExportDialog />
    <DataSourceDialog />
    <ImportWizard />
    <RecentProjectsDialog />
    <ProjectSettingsDialog />
    <NewProjectDialog />
  </div>
</template>
