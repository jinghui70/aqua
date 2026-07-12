// Tauri invoke 封装,统一错误处理。

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import type { Project, DbConfig, ValidationError } from "@/types/schema";

/** 调用 Tauri command,失败时弹错误提示。 */
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (err) {
    ElMessage.error(`操作失败: ${err}`);
    throw err;
  }
}

export function useTauri() {
  return {
    // 项目管理
    projectOpen: (path: string) => invoke<Project>("project_open", { path }),
    projectSave: (path: string, project: Project) =>
      invoke<void>("project_save", { path, project }),
    projectValidate: (project: Project) =>
      invoke<string>("project_validate", { project }),

    // 生成器
    generateDdl: (
      project: Project,
      dialect: string,
      opts?: { tables?: string[]; group?: string }
    ) =>
      invoke<string>("generate_ddl_command", {
        project,
        dialect,
        tables: opts?.tables,
        group: opts?.group,
      }),
    generateJava: (
      project: Project,
      table: string,
      opts?: { useLombok?: boolean; includeComment?: boolean; package?: string; className?: string }
    ) =>
      invoke<string>("generate_java_command", {
        project,
        table,
        useLombok: opts?.useLombok,
        includeComment: opts?.includeComment,
        package: opts?.package,
        className: opts?.className,
      }),
    generateFrontendJson: (project: Project, table: string) =>
      invoke<string>("generate_frontend_json_command", { project, table }),
    generateEnum: (project: Project, enumCode: string) =>
      invoke<string>("generate_enum_command", { project, enumCode }),
    generateStrConst: (
      project: Project,
      opts?: { group?: string; packageSuffix?: string; className?: string }
    ) =>
      invoke<string>("generate_strconst_command", {
        project,
        group: opts?.group,
        packageSuffix: opts?.packageSuffix,
        className: opts?.className,
      }),
    generateAlter: (oldProject: Project, newProject: Project, dialect: string) =>
      invoke<string>("generate_alter_command", { oldProject, newProject, dialect }),

    // 导入
    testConnection: (config: DbConfig) =>
      invoke<string>("test_connection_command", { config }),
    importFromDb: (config: DbConfig, basePackage?: string) =>
      invoke<Project>("import_from_db_command", { config, basePackage }),
    listTables: (config: DbConfig) =>
      invoke<string[]>("list_tables_command", { config }),
  };
}

export { invoke };
export type { ValidationError };
