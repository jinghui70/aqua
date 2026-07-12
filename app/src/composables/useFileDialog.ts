// OS 原生文件对话框封装(Tauri dialog plugin)。

import { open, save } from "@tauri-apps/plugin-dialog";

const JSON_FILTER = [{ name: "schema", extensions: ["json"] }];

/** 打开文件选择框,返回选中路径(取消返回 null)。 */
export async function pickOpenFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: JSON_FILTER,
  });
  return typeof result === "string" ? result : null;
}

/** 保存文件选择框,返回目标路径(取消返回 null)。 */
export async function pickSaveFile(
  defaultName = "schema.json"
): Promise<string | null> {
  const result = await save({
    defaultPath: defaultName,
    filters: JSON_FILTER,
  });
  return result ?? null;
}

const DATASET_FILTER = [{ name: "数据集", extensions: ["json", "db"] }];

/** 打开数据集文件(.json / .db)。 */
export async function pickDatasetOpen(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: DATASET_FILTER,
  });
  return typeof result === "string" ? result : null;
}

/** 保存数据集文件(.json / .db)。 */
export async function pickDatasetSave(
  defaultName = "dataset.json"
): Promise<string | null> {
  const result = await save({ defaultPath: defaultName, filters: DATASET_FILTER });
  return result ?? null;
}
