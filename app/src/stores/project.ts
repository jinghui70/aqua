// 项目状态(Pinia)。替代原 useProject composable。

import { defineStore } from "pinia";
import { ref } from "vue";
import type { Project, Table } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

/** 已打开的标签(表编辑 or 配置页)。 */
export interface OpenedTab {
  /** 唯一 key: 表用 "table:CODE",配置页用固定 key 如 "biztype" */
  key: string;
  /** 标签标题 */
  title: string;
  /** 路由路径 */
  path: string;
}

export const useProjectStore = defineStore("project", () => {
  const tauri = useTauri();

  const currentProject = ref<Project | null>(null);
  const currentPath = ref<string>("");
  const openedTabs = ref<OpenedTab[]>([]);
  const activeTab = ref<string>("");

  /** 新建空项目。 */
  function newProject() {
    currentProject.value = {
      version: "1.0.0",
      basePackage: "com.example",
      bizTypes: [],
      enums: [],
      groups: [{ code: "default", name: "默认分组" }],
      tables: [],
    };
    currentPath.value = "";
    openedTabs.value = [];
    activeTab.value = "";
  }

  /** 打开项目文件。 */
  async function openProject(path: string) {
    currentProject.value = await tauri.projectOpen(path);
    currentPath.value = path;
    openedTabs.value = [];
    activeTab.value = "";
  }

  /** 保存项目。 */
  async function saveProject(path?: string) {
    if (!currentProject.value) return;
    const target = path ?? currentPath.value;
    if (!target) throw new Error("未指定保存路径");
    await tauri.projectSave(target, currentProject.value);
    currentPath.value = target;
  }

  /** 打开一个标签(不重复)。返回路由路径。 */
  function openTab(tab: OpenedTab): string {
    if (!openedTabs.value.some((t) => t.key === tab.key)) {
      openedTabs.value.push(tab);
    }
    activeTab.value = tab.key;
    return tab.path;
  }

  /** 关闭标签,返回下一个应激活的路径(无则空)。 */
  function closeTab(key: string): string {
    const idx = openedTabs.value.findIndex((t) => t.key === key);
    if (idx < 0) return "";
    openedTabs.value.splice(idx, 1);
    if (activeTab.value === key) {
      const next = openedTabs.value[idx] ?? openedTabs.value[idx - 1];
      activeTab.value = next?.key ?? "";
      return next?.path ?? "/welcome";
    }
    return "";
  }

  /** 打开表编辑标签。 */
  function openTable(table: Table) {
    return openTab({
      key: `table:${table.code}`,
      title: table.code,
      path: `/table/${table.code}`,
    });
  }

  return {
    currentProject,
    currentPath,
    openedTabs,
    activeTab,
    newProject,
    openProject,
    saveProject,
    openTab,
    closeTab,
    openTable,
  };
});
