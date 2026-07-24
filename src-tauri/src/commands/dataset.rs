//! dataset 命令实现(数据集 .data JSONL 文件读写 + 目录扫描 + 新建)。

use aqua_core::dataset::{load_dataset, save_dataset, DatasetEntry, SchemaDiff};
use aqua_core::schema::Project;
use serde::Serialize;
use std::path::Path;

/// 数据集信息(目录扫描结果)。
#[derive(Serialize)]
pub struct DatasetInfo {
    pub name: String,
    pub path: String,
}

/// 加载结果: 重塑后的行数据 + 结构差异(非空表示数据集与项目结构不一致)。
#[derive(Serialize)]
pub struct LoadResult {
    pub entries: Vec<DatasetEntry>,
    pub diffs: Vec<SchemaDiff>,
}

/// Tauri command: 加载数据集文件(.data JSONL,按项目结构重塑,返回差异)。
#[tauri::command]
pub async fn dataset_load(path: String, project: Project) -> Result<LoadResult, String> {
    let (entries, diffs) = load_dataset(&path, &project).map_err(|e| e.to_string())?;
    Ok(LoadResult { entries, diffs })
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

/// Tauri command: 另存为时复制数据集({旧前缀}.*.data → {新前缀}.*.data)。
/// 数据集按 主文件名 命名,另存改了主文件名/目录 → 需把旧目录的 .data 复制到新位置,
/// 否则新项目扫不到数据集(bug: saveAs 数据集丢失)。返回复制的数据集数量。
#[tauri::command]
pub async fn copy_datasets(
    old_project_path: String,
    new_project_path: String,
) -> Result<usize, String> {
    if old_project_path == new_project_path {
        return Ok(0); // 普通保存(非另存),无需复制
    }
    let old = Path::new(&old_project_path);
    let old_dir = old.parent().ok_or("无效旧路径")?;
    let old_prefix = old.file_stem().and_then(|s| s.to_str()).ok_or("无效旧文件名")?;

    let new = Path::new(&new_project_path);
    let new_dir = new.parent().ok_or("无效新路径")?;
    let new_prefix = new.file_stem().and_then(|s| s.to_str()).ok_or("无效新文件名")?;

    let mut copied = 0;
    if let Ok(entries) = std::fs::read_dir(old_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let Some(name) = fname.to_str() else { continue };
            // 匹配 {old_prefix}.{dataset}.data,提取 dataset 名
            let Some(rest) = name.strip_prefix(&format!("{}.", old_prefix)) else { continue };
            let Some(dataset) = rest.strip_suffix(".data") else { continue };
            let dst = new_dir.join(format!("{}.{}.data", new_prefix, dataset));
            // 目标已存在则跳过,不覆盖(避免另存到已有数据集的项目时冲掉)
            if dst.exists() {
                continue;
            }
            std::fs::copy(entry.path(), &dst).map_err(|e| format!("复制数据集失败: {}", e))?;
            copied += 1;
        }
    }
    Ok(copied)
}
