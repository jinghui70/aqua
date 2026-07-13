// UI 状态(弹窗开关等)。
import { defineStore } from "pinia";
import { ref } from "vue";

export type ExportKind = "ddl" | "diff" | "strconst";

export const useUiStore = defineStore("ui", () => {
  // 导出弹窗
  const exportVisible = ref(false);
  const exportKind = ref<ExportKind>("ddl");

  function openExport(kind: ExportKind) {
    exportKind.value = kind;
    exportVisible.value = true;
  }

  // 数据源配置弹窗
  const dataSourceVisible = ref(false);
  function openDataSource() {
    dataSourceVisible.value = true;
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

  // 项目设置弹窗(中文名/basePackage)
  const projectSettingsVisible = ref(false);
  function openProjectSettings() {
    projectSettingsVisible.value = true;
  }

  return {
    exportVisible,
    exportKind,
    openExport,
    dataSourceVisible,
    openDataSource,
    importVisible,
    openImport,
    recentVisible,
    openRecent,
    projectSettingsVisible,
    openProjectSettings,
  };
});
