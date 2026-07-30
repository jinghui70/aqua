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
import DdlExportDialog from "@/components/DdlExportDialog.vue";
import DiffExportDialog from "@/components/DiffExportDialog.vue";
import StrConstExportDialog from "@/components/StrConstExportDialog.vue";
import DatabaseConfigDialog from "@/components/DatabaseConfigDialog.vue";
import ImportWizard from "@/components/ImportWizard.vue";
import RecentProjectsDialog from "@/components/RecentProjectsDialog.vue";
import NewProjectDialog from "@/components/NewProjectDialog.vue";
import ExitConfirmDialog from "@/components/ExitConfirmDialog.vue";

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
    <DdlExportDialog />
    <DiffExportDialog />
    <StrConstExportDialog />
    <DatabaseConfigDialog />
    <ImportWizard />
    <RecentProjectsDialog />
    <NewProjectDialog />
    <ExitConfirmDialog />

    <!-- 校验对话框(draggable) -->
    <el-dialog
      v-model="store.validateDialog.visible"
      :title="store.validateDialog.mode === 'confirm' ? '保存校验' : '校验提示'"
      draggable
      width="600px"
      :close-on-click-modal="false"
    >
      <div style="max-height:50vh;overflow:auto">
        {{ store.validateDialog.mode === 'confirm' ? '校验发现' : '项目存在' }} {{ store.validateDialog.errors.length }} 个问题:
        <div class="mt-8" v-html="store.formatErrorsHtml(store.validateDialog.errors)"></div>
      </div>
      <template #footer>
        <el-button v-if="store.validateDialog.mode === 'alert'" type="primary" @click="store.closeValidateDialog(false)">知道了</el-button>
        <template v-else>
          <el-button @click="store.closeValidateDialog(false)">取消去修</el-button>
          <el-button type="primary" @click="store.closeValidateDialog(true)">仍保存</el-button>
        </template>
      </template>
    </el-dialog>
  </div>
</template>
