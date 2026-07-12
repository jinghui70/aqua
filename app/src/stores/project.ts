// 项目状态(Pinia)。替代原 useProject composable。

import { defineStore } from "pinia";
import { ref } from "vue";
import type { Project, Table } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";
import { useRecentProjects } from "@/composables/useRecentProjects";

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
  const recent = useRecentProjects();

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
    recent.record(path);
  }

  /** 保存项目。 */
  async function saveProject(path?: string) {
    if (!currentProject.value) return;
    const target = path ?? currentPath.value;
    if (!target) throw new Error("未指定保存路径");
    await tauri.projectSave(target, currentProject.value);
    currentPath.value = target;
    recent.record(target);
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
      return next?.path ?? "/";
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

  // ===== 分组 CRUD =====

  function addGroup(code: string, name: string): string | null {
    if (!currentProject.value) return "无项目";
    if (currentProject.value.groups.some((g) => g.code === code)) {
      return `分组 ${code} 已存在`;
    }
    currentProject.value.groups.push({ code, name });
    return null;
  }

  function renameGroup(code: string, name: string) {
    const g = currentProject.value?.groups.find((g) => g.code === code);
    if (g) g.name = name;
  }

  /** 删除分组(仅当无表引用)。返回错误信息或 null。 */
  function deleteGroup(code: string): string | null {
    if (!currentProject.value) return "无项目";
    const used = currentProject.value.tables.some((t) => t.group === code);
    if (used) return "分组下还有表,无法删除";
    const idx = currentProject.value.groups.findIndex((g) => g.code === code);
    if (idx >= 0) currentProject.value.groups.splice(idx, 1);
    return null;
  }

  // ===== 表 CRUD =====

  function addTable(code: string, name: string, group: string): string | null {
    if (!currentProject.value) return "无项目";
    if (currentProject.value.tables.some((t) => t.code === code)) {
      return `表 ${code} 已存在`;
    }
    currentProject.value.tables.push({ code, name, group, fields: [] });
    return null;
  }

  function renameTable(code: string, name: string) {
    const t = currentProject.value?.tables.find((t) => t.code === code);
    if (t) t.name = name;
  }

  function deleteTable(code: string) {
    if (!currentProject.value) return;
    const idx = currentProject.value.tables.findIndex((t) => t.code === code);
    if (idx >= 0) currentProject.value.tables.splice(idx, 1);
    // 关闭对应标签
    closeTab(`table:${code}`);
  }

  /**
   * 合并导入的表进当前项目。
   * @param imported 导入的完整 Project
   * @param tableCodes 选中要导入的表 code
   * @param group 目标分组
   * @param overwrite 同名表 true=覆盖 false=跳过
   */
  function mergeImportedTables(
    imported: Project,
    tableCodes: string[],
    group: string,
    overwrite: boolean
  ): { added: number; skipped: number } {
    if (!currentProject.value) return { added: 0, skipped: 0 };
    let added = 0;
    let skipped = 0;
    for (const code of tableCodes) {
      const src = imported.tables.find((t) => t.code === code);
      if (!src) continue;
      const table = { ...src, group };
      const existIdx = currentProject.value.tables.findIndex(
        (t) => t.code === code
      );
      if (existIdx >= 0) {
        if (overwrite) {
          currentProject.value.tables[existIdx] = table;
          added++;
        } else {
          skipped++;
        }
      } else {
        currentProject.value.tables.push(table);
        added++;
      }
    }
    return { added, skipped };
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
    addGroup,
    renameGroup,
    deleteGroup,
    addTable,
    renameTable,
    deleteTable,
    mergeImportedTables,
  };
});
