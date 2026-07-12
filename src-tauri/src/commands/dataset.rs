//! dataset 命令实现(数据集文件读写)。

use aqua_core::dataset::{load_dataset, save_dataset, DatasetEntry};
use aqua_core::schema::Project;

/// Tauri command: 加载数据集文件(.json / .db),按项目表结构校验后返回条目。
#[tauri::command]
pub async fn dataset_load(
    path: String,
    project: Project,
) -> Result<Vec<DatasetEntry>, String> {
    load_dataset(&path, &project).map_err(|e| e.to_string())
}

/// Tauri command: 保存数据集条目到文件(按扩展名分派格式)。
#[tauri::command]
pub async fn dataset_save(
    path: String,
    project: Project,
    entries: Vec<DatasetEntry>,
) -> Result<(), String> {
    save_dataset(&path, &project, &entries).map_err(|e| e.to_string())
}
