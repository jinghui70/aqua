// 项目状态(Pinia)。替代原 useProject composable。

import { acceptHMRUpdate, defineStore } from "pinia";
import { nextTick, ref, watch } from "vue";
import { useUiStore } from "@/stores/ui";
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
  /** 唯一 key: 表用 "table:ID",配置页用固定 key 如 "biztype" */
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
  /** 项目只读(打开默认只读防误改,新建可编辑,工具栏加/解锁切换)。 */
  const readOnly = ref(false);
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
      autoGenStrategies: [],
      groups: [{ code: "default", name: "默认分组" }],
      tables: [],
    };
    currentPath.value = "";
    openedTabs.value = [];
    activeTab.value = "";
    dirty.value = false;
    readOnly.value = false;
    void datasource.load("");
    void nextTick(() => {
      suppressDirty = false;
    });
  }

  /** 打开项目文件。调用方应先 confirmIfDirty。 */
  async function openProject(path: string) {
    suppressDirty = true;
    const p = await tauri.projectOpen(path);
    p.tables.forEach((t) => (t.id = crypto.randomUUID()));
    currentProject.value = p;
    currentPath.value = path;
    openedTabs.value = [];
    activeTab.value = "";
    dirty.value = false;
    readOnly.value = true; // 打开已有项目默认只读
    recent.record(path, currentProject.value?.name ?? undefined);
    await datasource.load(path);
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
    const pathChanged = target !== datasource.projectPath;
    currentPath.value = target;
    recent.record(target, currentProject.value.name ?? undefined);
    dirty.value = false;
    // 更新 .gitignore
    try {
      await tauri.updateGitignore(target);
    } catch (err) {
      console.warn('更新 .gitignore 失败:', err);
    }
    // 首次保存或另存为:把内存态数据源落盘到该路径对应配置文件
    if (pathChanged) await datasource.bindDirAndPersist(target);
  }

  /**
   * 有未保存改动时提醒。返回 true 表示可继续(已保存或丢弃),false 表示用户取消。
   * 选"保存"且无当前路径时弹另存框。
   */
  async function confirmIfDirty(): Promise<boolean> {
    if (!dirty.value) return true;
    const ui = useUiStore();
    const action = await ui.openExitConfirm();
    if (action === "cancel") return false;
    if (action === "discard") return true;
    // save
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
    readOnly.value = false;
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
      key: `table:${table.id}`,
      title: table.code,
      path: `/table/${table.id}`,
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

  /** 更新分组:可改 code 与名称;改 code 时级联更新所有表的 group 引用。返回错误或 null。 */
  function updateGroup(
    oldCode: string,
    newCode: string,
    name: string
  ): string | null {
    const p = currentProject.value;
    if (!p) return "无项目";
    newCode = newCode.trim();
    if (!newCode) return "code 不能为空";
    if (newCode !== oldCode && p.groups.some((g) => g.code === newCode)) {
      return `分组 ${newCode} 已存在`;
    }
    const g = p.groups.find((g) => g.code === oldCode);
    if (!g) return `分组 ${oldCode} 不存在`;
    if (newCode !== oldCode) {
      g.code = newCode;
      for (const t of p.tables) {
        if (t.group === oldCode) t.group = newCode;
      }
    }
    g.name = name;
    return null;
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

  function addTable(code: string, name: string, group: string, comment?: string): string | null {
    if (!currentProject.value) return "无项目";
    if (currentProject.value.tables.some((t) => t.code === code)) {
      return `表 ${code} 已存在`;
    }
    currentProject.value.tables.push({ id: crypto.randomUUID(), code, name, group, fields: [], comment: comment || undefined });
    return null;
  }

  /** 更新表:可改 code 与名称;页签用 id 标识,改 code 不影响页签。返回错误或 null。 */
  function updateTable(id: string, code: string, name: string, comment?: string): string | null {
    const p = currentProject.value;
    if (!p) return "无项目";
    code = code.trim().toUpperCase();
    if (!code) return "code 不能为空";
    if (p.tables.some((t) => t.id !== id && t.code === code)) {
      return `表 ${code} 已存在`;
    }
    const t = p.tables.find((t) => t.id === id);
    if (!t) return "表不存在";
    t.code = code;
    t.name = name;
    t.comment = comment || undefined;
    // 页签 key/path 用 id 不变,但 title 用 code 展示,需同步
    const tab = openedTabs.value.find((x) => x.key === `table:${id}`);
    if (tab) tab.title = code;
    return null;
  }

  function deleteTable(code: string): string {
    const p = currentProject.value;
    if (!p) return "";
    const idx = p.tables.findIndex((t) => t.code === code);
    if (idx < 0) return "";
    const id = p.tables[idx].id;
    p.tables.splice(idx, 1);
    // 关闭对应标签(key 用 id,与 openTable 一致);返回下一个应跳转的路径(删的是当前页签时非空)
    return closeTab(`table:${id}`);
  }

  // ===== 字段级联索引(索引 fields 存 field code,字段删/改需同步)=====

  /** 字段 code 改名后,同步该表索引中引用旧 code 的字段。 */
  function renameFieldCode(tableId: string, oldCode: string, newCode: string) {
    const t = currentProject.value?.tables.find((x) => x.id === tableId);
    if (!t?.indexes) return;
    for (const ix of t.indexes) {
      for (const f of ix.fields) {
        if (f.code === oldCode) f.code = newCode;
      }
    }
  }

  /** 字段删除后,从该表索引中移除引用该 code 的字段。 */
  function removeFieldFromIndexes(tableId: string, code: string) {
    const t = currentProject.value?.tables.find((x) => x.id === tableId);
    if (!t?.indexes) return;
    for (const ix of t.indexes) {
      ix.fields = ix.fields.filter((f) => f.code !== code);
    }
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
      // 导入表来自反解(后端 Table 无运行时 id),须生成新 id,否则页签/route key 缺失
      const table = { ...src, id: crypto.randomUUID(), group };
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
      id: crypto.randomUUID(),
      code: newCode,
      name: `${src.name}(副本)`,
      group: src.group,
      fields: JSON.parse(JSON.stringify(src.fields)),
      indexes: src.indexes ? JSON.parse(JSON.stringify(src.indexes)) : undefined,
      comment: src.comment,
    };
    p.tables.push(copy);
    return newCode;
  }

  /** 切换项目只读/可编辑(工具栏加/解锁)。 */
  function toggleReadOnly() {
    readOnly.value = !readOnly.value;
  }

  return {
    currentProject,
    currentPath,
    openedTabs,
    activeTab,
    dirty,
    readOnly,
    toggleReadOnly,
    newProject,
    openProject,
    saveProject,
    confirmIfDirty,
    closeProject,
    openTab,
    closeTab,
    openTable,
    addGroup,
    updateGroup,
    deleteGroup,
    addTable,
    updateTable,
    deleteTable,
    renameFieldCode,
    removeFieldFromIndexes,
    reorderGroups,
    moveTable,
    duplicateTable,
    mergeImportedTables,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useProjectStore, import.meta.hot));
}
