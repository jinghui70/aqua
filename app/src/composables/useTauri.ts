// Tauri invoke 封装,统一错误处理。

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import type { Project, DbConfig, DatabaseInfo, ValidationError, DatasetEntry, BizTypeDefine } from "@/types/schema";

/** 表信息(表名 + 注释),listTables 返回,对齐 Rust TableInfo。 */
export interface TableInfo {
  name: string;
  comment: string | null;
}

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
    updateGitignore: (projectPath: string) =>
      invoke<void>("update_gitignore", { projectPath }),

    // 数据集
    datasetLoad: (path: string, project: Project) =>
      invoke<DatasetEntry[]>("dataset_load", { path, project }),
    datasetSave: (path: string, project: Project, entries: DatasetEntry[]) =>
      invoke<void>("dataset_save", { path, project, entries }),

    // 数据源持久化(密码 AES 加密)
    datasourceLoad: (projectPath: string) =>
      invoke<Array<[string, DbConfig]>>("datasource_load", { projectPath }),
    datasourceSave: (projectPath: string, sources: Array<[string, DbConfig]>) =>
      invoke<void>("datasource_save", { projectPath, sources }),

    // 内置业务类型清单(打包资源文件)
    builtinBiztypesLoad: () => invoke<BizTypeDefine[]>("builtin_biztypes_load"),

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
    importFromDb: (config: DbConfig, tables: TableInfo[], basePackage?: string) =>
      invoke<Project>("import_from_db_command", { config, tables, basePackage }),
    listTables: (config: DbConfig) =>
      invoke<TableInfo[]>("list_tables_command", { config }),

    // 数据库支持管理(drivers/databases.json)
    listDatabases: () => invoke<DatabaseInfo[]>("list_databases"),
    installDriver: (dialect: string, jarPath: string) =>
      invoke<void>("install_driver", { dialect, jarPath }),
    uninstallDriver: (dialect: string) =>
      invoke<void>("uninstall_driver", { dialect }),
  };
}

export { invoke };
export type { ValidationError };
