// 项目状态管理(全局单例)。

import { ref, computed } from "vue";
import type { Project, Table } from "@/types/schema";
import { useTauri } from "@/composables/useTauri";

const currentProject = ref<Project | null>(null);
const currentPath = ref<string>("");
const selectedTableCode = ref<string>("");

export function useProject() {
  const tauri = useTauri();

  const currentTable = computed<Table | null>(() => {
    if (!currentProject.value || !selectedTableCode.value) return null;
    return (
      currentProject.value.tables.find((t) => t.code === selectedTableCode.value) ?? null
    );
  });

  async function openProject(path: string) {
    currentProject.value = await tauri.projectOpen(path);
    currentPath.value = path;
    selectedTableCode.value =
      currentProject.value.tables[0]?.code ?? "";
  }

  async function saveProject(path?: string) {
    if (!currentProject.value) return;
    const target = path ?? currentPath.value;
    if (!target) {
      throw new Error("未指定保存路径");
    }
    await tauri.projectSave(target, currentProject.value);
    currentPath.value = target;
  }

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
    selectedTableCode.value = "";
  }

  function selectTable(code: string) {
    selectedTableCode.value = code;
  }

  return {
    currentProject,
    currentPath,
    selectedTableCode,
    currentTable,
    openProject,
    saveProject,
    newProject,
    selectTable,
  };
}
