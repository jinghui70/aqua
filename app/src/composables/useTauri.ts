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
    generateDdl: (project: Project, dialect: string) =>
      invoke<string>("generate_ddl_command", { project, dialect }),
    generateJava: (
      project: Project,
      table: string,
      opts?: { useLombok?: boolean; package?: string; className?: string }
    ) =>
      invoke<string>("generate_java_command", {
        project,
        table,
        useLombok: opts?.useLombok,
        package: opts?.package,
        className: opts?.className,
      }),
    generateFrontendJson: (project: Project, table: string) =>
      invoke<string>("generate_frontend_json_command", { project, table }),
    generateEnum: (project: Project, enumCode: string) =>
      invoke<string>("generate_enum_command", { project, enumCode }),

    // 导入
    testConnection: (config: DbConfig) =>
      invoke<string>("test_connection_command", { config }),
    importFromDb: (config: DbConfig, basePackage?: string) =>
      invoke<Project>("import_from_db_command", { config, basePackage }),
  };
}

export { invoke };
export type { ValidationError };
