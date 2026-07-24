//! 数据集导入导出命令(dataset <-> 数据库)。

use aqua_core::dataset::{load_dataset, save_dataset};
use aqua_core::driver::{create_driver, DbConfig};
use aqua_core::schema::Project;
use serde::Serialize;
use std::collections::HashSet;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

use super::database::drivers_dir;

/// 定位 bundle 内 connector.jar 绝对路径。
fn connector_jar_path<R: Runtime>(app: &AppHandle<R>) -> String {
    app.path()
        .resolve("resources/connector.jar", BaseDirectory::Resource)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "connector.jar".to_string())
}

#[derive(Serialize)]
pub struct ImportResult {
    pub total: usize,
}

#[derive(Serialize)]
pub struct ExportResult {
    pub affected: usize,
}

/// Tauri command: 从数据库导入数据到数据集(.data JSONL)。
/// tables 为 None -> 全表;Some -> 仅导入选中表(未选中表保留原数据)。
#[tauri::command]
pub async fn dataset_import<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    project: Project,
    config: DbConfig,
    tables: Option<Vec<String>>,
) -> Result<ImportResult, String> {
    let drivers = drivers_dir(&app).ok();
    let connector = connector_jar_path(&app);
    let driver = create_driver(config, drivers, &connector).map_err(|e| e.to_string())?;
    // 加载原数据集,保留未选中表的数据
    let mut entries = load_dataset(&path, &project).map_err(|e| e.to_string())?;
    let want: HashSet<String> = match tables {
        Some(ts) => ts.into_iter().collect(),
        None => project.tables.iter().map(|t| t.code.clone()).collect(),
    };
    let mut total = 0;
    for entry in &mut entries {
        if !want.contains(&entry.table) {
            continue;
        }
        let rows = driver
            .query_table_rows(&entry.table)
            .await
            .map_err(|e| e.to_string())?;
        total += rows.len();
        entry.data = rows;
    }
    save_dataset(&path, &project, &entries).map_err(|e| e.to_string())?;
    Ok(ImportResult { total })
}

/// Tauri command: 从数据集导出数据到数据库(TRUNCATE + INSERT)。
/// tables 为 None -> 全表;Some -> 仅导出选中表。
#[tauri::command]
pub async fn dataset_export<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    project: Project,
    config: DbConfig,
    truncate: bool,
    tables: Option<Vec<String>>,
) -> Result<ExportResult, String> {
    let entries = load_dataset(&path, &project).map_err(|e| e.to_string())?;
    let want: HashSet<String> = match tables {
        Some(ts) => ts.into_iter().collect(),
        None => entries.iter().map(|e| e.table.clone()).collect(),
    };
    let drivers = drivers_dir(&app).ok();
    let connector = connector_jar_path(&app);
    let driver = create_driver(config, drivers, &connector).map_err(|e| e.to_string())?;
    let mut affected = 0;
    for entry in entries {
        if !want.contains(&entry.table) {
            continue;
        }
        if truncate {
            let sql = format!("TRUNCATE TABLE {}", entry.table);
            driver.execute_update(&sql).await.map_err(|e| e.to_string())?;
        }
        // INSERT 分批(100 行)
        for chunk in entry.data.chunks(100) {
            let table = project
                .tables
                .iter()
                .find(|t| t.code == entry.table)
                .ok_or("表不存在")?;
            let fields: Vec<_> = table.fields.iter().map(|f| f.code.as_str()).collect();
            let values: Vec<String> = chunk
                .iter()
                .map(|row| {
                    let vals: Vec<String> = fields
                        .iter()
                        .map(|f| {
                            let key = f.to_uppercase();
                            match row.get(&key) {
                                Some(v) if v.is_null() => "NULL".to_string(),
                                Some(v) => format!("'{}'", v.as_str().unwrap_or("").replace('\'', "''")),
                                None => "NULL".to_string(),
                            }
                        })
                        .collect();
                    format!("({})", vals.join(","))
                })
                .collect();
            let sql = format!(
                "INSERT INTO {} ({}) VALUES {}",
                entry.table,
                fields.join(","),
                values.join(",")
            );
            let n = driver.execute_update(&sql).await.map_err(|e| e.to_string())?;
            affected += n;
        }
    }
    Ok(ExportResult { affected })
}
