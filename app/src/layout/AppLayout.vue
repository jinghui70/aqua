<script setup lang="ts">
// 布局分两态:
// - 无项目: 全屏欢迎页(独立,无左树)
// - 有项目: 左分组树 + 多标签工作区 + 状态栏
// 菜单是原生窗口菜单(Rust 侧)。
import { useProjectStore } from "@/stores/project";
import Welcome from "@/views/Welcome.vue";
import GroupTreeAside from "./GroupTreeAside.vue";
import TabWorkspace from "./TabWorkspace.vue";
import StatusBar from "./StatusBar.vue";
import ExportDialog from "@/components/ExportDialog.vue";
import DataSourceDialog from "@/components/DataSourceDialog.vue";
import ImportWizard from "@/components/ImportWizard.vue";

const store = useProjectStore();
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <!-- 无项目: 全屏欢迎页 -->
    <Welcome v-if="!store.currentProject" class="flex-1 overflow-auto" />

    <!-- 有项目: 工作区布局 -->
    <template v-else>
      <div class="flex-1 min-h-0 flex overflow-hidden">
        <aside class="w-260 border-r border-gray-200 flex-shrink-0">
          <GroupTreeAside />
        </aside>
        <main class="flex-1 min-w-0 overflow-hidden">
          <TabWorkspace />
        </main>
      </div>
      <StatusBar />
    </template>

    <!-- 全局弹窗(两态都可用) -->
    <ExportDialog />
    <DataSourceDialog />
    <ImportWizard />
  </div>
</template>
