// UI 状态(弹窗开关等)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, ref } from "vue";

export type ExportKind = "ddl" | "diff" | "strconst";

export const useUiStore = defineStore("ui", () => {
  // 导出弹窗
  const exportVisible = ref(false);
  const exportKind = ref<ExportKind>("ddl");

  function openExport(kind: ExportKind) {
    exportKind.value = kind;
    exportVisible.value = true;
  }

  // 数据库配置弹窗(驱动安装/显隐)
  const databaseConfigVisible = ref(false);
  function openDatabaseConfig() {
    databaseConfigVisible.value = true;
  }

  // 导入向导弹窗
  const importVisible = ref(false);
  function openImport() {
    importVisible.value = true;
  }

  // 最近项目弹窗
  const recentVisible = ref(false);
  function openRecent() {
    recentVisible.value = true;
  }

  // 新建项目弹窗(中文名 + basePackage)
  const newProjectVisible = ref(false);
  function openNewProject() {
    newProjectVisible.value = true;
  }

  // 是否有对话框打开(菜单事件据此忽略,避免操作被打断;原生菜单栏无法灰显)
  const anyDialogOpen = computed(
    () =>
      exportVisible.value ||
      databaseConfigVisible.value ||
      importVisible.value ||
      recentVisible.value ||
      newProjectVisible.value
  );

  return {
    exportVisible,
    exportKind,
    openExport,
    databaseConfigVisible,
    openDatabaseConfig,
    importVisible,
    openImport,
    recentVisible,
    openRecent,
    newProjectVisible,
    openNewProject,
    anyDialogOpen,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUiStore, import.meta.hot));
}
