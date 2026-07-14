// 项目状态(Pinia)。替代原 useProject composable。

import { defineStore } from "pinia";
import { nextTick, ref, watch } from "vue";
import { ElMessageBox } from "element-plus";
import type { Project, Table } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";
import { useRecentProjects } from "@/composables/useRecentProjects";
import { useDataSourceStore } from "@/stores/datasource";
import { pickSaveFile } from "@/composables/useFileDialog";

/** 取文件所在目录(兼容 / 与 \\)。无分隔符返回空串。 */
function dirOf(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx >= 0 ? path.slice(0, idx) : "";
}

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
  const datasource = useDataSourceStore();

  const currentProject = ref<Project | null>(null);
  const currentPath = ref<string>("");
  const openedTabs = ref<OpenedTab[]>([]);
  const activeTab = ref<string>("");
  /** 有未保存改动(任何对 currentProject 的深层变动)。 */
  const dirty = ref(false);
  // 加载/新建/关闭时短暂抑制 dirty watch,避免赋值本身被记为改动。
  let suppressDirty = false;

  watch(
    currentProject,
    () => {
      if (!suppressDirty) dirty.value = true;
    },
    { deep: true }
  );

  /** 新建空项目。调用方应先 confirmIfDirty。 */
  function newProject(name: string, basePackage: string) {
    suppressDirty = true;
    currentProject.value = {
      version: "1.0.0",
      basePackage: basePackage.trim() || "com.example",
      name: name.trim() || undefined,
      bizTypes: [],
      enums: [],
      groups: [{ code: "default", name: "默认分组" }],
      tables: [],
    };
    currentPath.value = "";
    openedTabs.value = [];
    activeTab.value = "";
    dirty.value = false;
    void datasource.load("");
    void nextTick(() => {
      suppressDirty = false;
    });
  }

  /** 打开项目文件。调用方应先 confirmIfDirty。 */
  async function openProject(path: string) {
    suppressDirty = true;
    currentProject.value = await tauri.projectOpen(path);
    currentPath.value = path;
    openedTabs.value = [];
    activeTab.value = "";
    dirty.value = false;
    recent.record(path, currentProject.value?.name ?? undefined);
    await datasource.load(dirOf(path));
    void nextTick(() => {
      suppressDirty = false;
    });
  }

  /** 保存项目。 */
  async function saveProject(path?: string) {
    if (!currentProject.value) return;
    const target = path ?? currentPath.value;
    if (!target) throw new Error("未指定保存路径");
    await tauri.projectSave(target, currentProject.value);
    const firstBind = dirOf(target) !== datasource.projectDir;
    currentPath.value = target;
    recent.record(target, currentProject.value.name ?? undefined);
    dirty.value = false;
    // 首次保存/另存到新目录:把内存态数据源落盘到该目录
    if (firstBind) await datasource.bindDirAndPersist(dirOf(target));
  }

  /**
   * 有未保存改动时提醒。返回 true 表示可继续(已保存或丢弃),false 表示用户取消。
   * 选"保存"且无当前路径时弹另存框。
   */
  async function confirmIfDirty(): Promise<boolean> {
    if (!dirty.value) return true;
    let action: "save" | "discard" | "cancel";
    try {
      await ElMessageBox.confirm("有未保存改动,是否保存?", "提示", {
        distinguishCancelAndClose: true,
        confirmButtonText: "保存",
        cancelButtonText: "不保存",
        type: "warning",
      });
      action = "save";
    } catch (e) {
      action = e === "cancel" ? "discard" : "cancel";
    }
    if (action === "cancel") return false;
    if (action === "discard") return true;
    let target = currentPath.value;
    if (!target) {
      target = (await pickSaveFile()) ?? "";
      if (!target) return false;
    }
    await saveProject(target);
    return true;
  }

  /** 关闭项目回欢迎页。有未保存改动先提醒;取消则不关闭。 */
  async function closeProject(): Promise<boolean> {
    if (!(await confirmIfDirty())) return false;
    suppressDirty = true;
    currentProject.value = null;
    currentPath.value = "";
    openedTabs.value = [];
    activeTab.value = "";
    dirty.value = false;
    void datasource.load("");
    void nextTick(() => {
      suppressDirty = false;
    });
    return true;
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

  // ===== 拖拽:分组重排 / 表移动 =====

  /** 分组重排:把 fromCode 移到 toCode 的前/后。 */
  function reorderGroups(
    fromCode: string,
    toCode: string,
    type: "before" | "after"
  ) {
    const p = currentProject.value;
    if (!p) return;
    const fromIdx = p.groups.findIndex((g) => g.code === fromCode);
    if (fromIdx < 0) return;
    const [g] = p.groups.splice(fromIdx, 1);
    let toIdx = p.groups.findIndex((x) => x.code === toCode);
    if (toIdx < 0) {
      p.groups.push(g);
      return;
    }
    if (type === "after") toIdx++;
    p.groups.splice(toIdx, 0, g);
  }

  /**
   * 表移动到目标分组。ref 省略则加到分组末尾;
   * 给定 ref 则插到参考表前/后(可跨分组)。
   */
  function moveTable(
    code: string,
    toGroup: string,
    ref?: { refCode: string; type: "before" | "after" }
  ) {
    const p = currentProject.value;
    if (!p) return;
    const fromIdx = p.tables.findIndex((t) => t.code === code);
    if (fromIdx < 0) return;
    const [t] = p.tables.splice(fromIdx, 1);
    t.group = toGroup;
    if (!ref) {
      p.tables.push(t);
      return;
    }
    let toIdx = p.tables.findIndex((x) => x.code === ref.refCode);
    if (toIdx < 0) {
      p.tables.push(t);
      return;
    }
    if (ref.type === "after") toIdx++;
    p.tables.splice(toIdx, 0, t);
  }

  /** 复制表结构为新表(code 加 _COPY 后缀递增,name 加(副本),字段索引深拷贝)。返回新 code,失败返回空串。 */
  function duplicateTable(code: string): string {
    const p = currentProject.value;
    if (!p) return "";
    const src = p.tables.find((t) => t.code === code);
    if (!src) return "";
    let newCode = `${code}_COPY`;
    let n = 2;
    while (p.tables.some((t) => t.code === newCode)) {
      newCode = `${code}_COPY${n++}`;
    }
    const copy: Table = {
      code: newCode,
      name: `${src.name}(副本)`,
      group: src.group,
      fields: structuredClone(src.fields),
      indexes: src.indexes ? structuredClone(src.indexes) : undefined,
      comment: src.comment,
    };
    p.tables.push(copy);
    return newCode;
  }

  return {
    currentProject,
    currentPath,
    openedTabs,
    activeTab,
    dirty,
    newProject,
    openProject,
    saveProject,
    confirmIfDirty,
    closeProject,
    openTab,
    closeTab,
    openTable,
    addGroup,
    renameGroup,
    deleteGroup,
    addTable,
    renameTable,
    deleteTable,
    reorderGroups,
    moveTable,
    duplicateTable,
    mergeImportedTables,
  };
});
