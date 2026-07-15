//! 数据源持久化 Tauri commands。
//!
//! 密钥落在平台 app data dir 下的 `key`;aqua-core 不感知平台目录,由此层解析后传入。

use aqua_core::datasource::{load_db_config, save_db_config, DataSourceConfig};
use tauri::{AppHandle, Manager, Runtime};

/// 解析平台 app data dir 下的密钥文件路径,必要时创建目录。
fn key_path<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建应用数据目录失败: {}", e))?;
    let key = dir.join("key");
    key.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "密钥路径含非法字符".to_string())
}

/// 加载项目对应的数据源配置(解密密码)。
#[tauri::command]
pub async fn datasource_load<R: Runtime>(
    app: AppHandle<R>,
    project_path: String,
) -> Result<Vec<(String, DataSourceConfig)>, String> {
    let key = key_path(&app)?;
    load_db_config(&project_path, &key).map_err(|e| e.to_string())
}

/// 保存数据源配置到项目对应的配置文件(加密密码)。
#[tauri::command]
pub async fn datasource_save<R: Runtime>(
    app: AppHandle<R>,
    project_path: String,
    sources: Vec<(String, DataSourceConfig)>,
) -> Result<(), String> {
    let key = key_path(&app)?;
    save_db_config(&project_path, &key, sources).map_err(|e| e.to_string())
}
