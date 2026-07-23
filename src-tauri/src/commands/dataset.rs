//! dataset 命令实现(数据集 .data JSONL 文件读写 + 目录扫描 + 新建)。

use aqua_core::dataset::{load_dataset, save_dataset, DatasetEntry};
use aqua_core::schema::Project;
use serde::Serialize;
use std::path::Path;

/// 数据集信息(目录扫描结果)。
#[derive(Serialize)]
pub struct DatasetInfo {
    pub name: String,
    pub path: String,
}

/// Tauri command: 加载数据集文件(.data JSONL)。
#[tauri::command]
pub async fn dataset_load(path: String, project: Project) -> Result<Vec<DatasetEntry>, String> {
    load_dataset(&path, &project).map_err(|e| e.to_string())
}

/// Tauri command: 保存数据集到 .data JSONL(按主键排序)。
#[tauri::command]
pub async fn dataset_save(
    path: String,
    project: Project,
    entries: Vec<DatasetEntry>,
) -> Result<(), String> {
    save_dataset(&path, &project, &entries).map_err(|e| e.to_string())
}

/// Tauri command: 扫描项目目录的数据集文件({前缀}.*.data)。
#[tauri::command]
pub async fn scan_datasets(project_path: String) -> Result<Vec<DatasetInfo>, String> {
    let path = Path::new(&project_path);
    let dir = path.parent().ok_or("无效项目路径")?;
    let prefix = path.file_stem().and_then(|s| s.to_str()).ok_or("无效文件名")?;

    let mut datasets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(rest) = name.strip_prefix(&format!("{}.", prefix)) {
                    if let Some(dataset_name) = rest.strip_suffix(".data") {
                        datasets.push(DatasetInfo {
                            name: dataset_name.to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }
    Ok(datasets)
}

/// Tauri command: 新建空数据集文件({前缀}.{name}.data)。
#[tauri::command]
pub async fn create_dataset(project_path: String, name: String) -> Result<String, String> {
    let path = Path::new(&project_path);
    let dir = path.parent().ok_or("无效项目路径")?;
    let prefix = path.file_stem().and_then(|s| s.to_str()).ok_or("无效文件名")?;
    let file_name = format!("{}.{}.data", prefix, name);
    let file_path = dir.join(&file_name);
    std::fs::write(&file_path, "").map_err(|e| format!("创建失败: {}", e))?;
    Ok(file_path.to_string_lossy().to_string())
}
