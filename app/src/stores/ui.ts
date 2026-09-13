// UI 状态(弹窗开关等)。
import { acceptHMRUpdate, defineStore } from "pinia";
import { computed, ref } from "vue";

export type ExportKind = "ddl" | "diff" | "strconst";

/** 设置对话框分类(与 SettingsDialog 左侧导航对应)。 */
export type SettingsPanel = "settings" | "datasource" | "biztype" | "strategy" | "driver";

export const useUiStore = defineStore("ui", () => {
  // 导出弹窗(三个独立)
  const ddlExportVisible = ref(false);
  const diffExportVisible = ref(false);
  const strConstExportVisible = ref(false);

  function openExport(kind: ExportKind) {
    if (kind === "ddl") ddlExportVisible.value = true;
    else if (kind === "diff") diffExportVisible.value = true;
    else strConstExportVisible.value = true;
  }

  // 设置对话框(项目设置/数据源/业务类型/自动生成策略/驱动管理;Cmd+, / 菜单打开)
  const settingsVisible = ref(false);
  const settingsPanel = ref<SettingsPanel>("settings");
  function openSettings(panel: SettingsPanel = "settings") {
    settingsPanel.value = panel;
    settingsVisible.value = true;
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

  // 退出/关闭确认(保存/不保存/取消,三按钮 ElDialog)
  const exitConfirmVisible = ref(false);
  let exitConfirmResolve: ((v: "save" | "discard" | "cancel") => void) | null = null;
  function openExitConfirm(): Promise<"save" | "discard" | "cancel"> {
    return new Promise((resolve) => {
      exitConfirmResolve = resolve;
      exitConfirmVisible.value = true;
    });
  }
  function resolveExitConfirm(v: "save" | "discard" | "cancel") {
    exitConfirmVisible.value = false;
    exitConfirmResolve?.(v);
    exitConfirmResolve = null;
  }

  // 是否有对话框打开(菜单事件据此忽略,避免操作被打断;原生菜单栏无法灰显)
  const anyDialogOpen = computed(
    () =>
      ddlExportVisible.value ||
      diffExportVisible.value ||
      strConstExportVisible.value ||
      settingsVisible.value ||
      importVisible.value ||
      recentVisible.value ||
      newProjectVisible.value ||
      exitConfirmVisible.value
  );

  return {
    ddlExportVisible,
    diffExportVisible,
    strConstExportVisible,
    openExport,
    settingsVisible,
    settingsPanel,
    openSettings,
    importVisible,
    openImport,
    recentVisible,
    openRecent,
    newProjectVisible,
    openNewProject,
    exitConfirmVisible,
    openExitConfirm,
    resolveExitConfirm,
    anyDialogOpen,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUiStore, import.meta.hot));
}
