<script setup lang="ts">
// 主布局:左分组树 + 右多标签工作区 + 底部状态栏。
// 菜单是原生窗口菜单(Rust 侧),此处无菜单栏。
import GroupTreeAside from "./GroupTreeAside.vue";
import TabWorkspace from "./TabWorkspace.vue";
import StatusBar from "./StatusBar.vue";
import ExportDialog from "@/components/ExportDialog.vue";
</script>

<template>
  <div class="h-screen flex flex-col">
    <div class="flex-1 flex overflow-hidden">
      <aside class="w-260 border-r border-gray-200 flex-shrink-0">
        <GroupTreeAside />
      </aside>
      <main class="flex-1 overflow-hidden">
        <TabWorkspace />
      </main>
    </div>
    <StatusBar />
    <!-- 全局弹窗 -->
    <ExportDialog />
  </div>
</template>
