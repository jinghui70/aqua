//! 数据库支持管理 Tauri commands(drivers/databases.json)。
//!
//! 应用级状态:隐藏的数据库 + 已装 JDBC 驱动。落在 app_data_dir/drivers/。

use aqua_core::driver::state::{self, DatabaseInfo};
use tauri::{AppHandle, Manager, Runtime};

/// 解析 app_data_dir/drivers/ 路径,必要时创建。
pub fn drivers_dir<R: Runtime>(app: &AppHandle<R>) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    let drivers = dir.join("drivers");
    std::fs::create_dir_all(&drivers).map_err(|e| format!("创建 drivers 目录失败: {}", e))?;
    Ok(drivers)
}

/// 列出全部数据库(清单 + hidden/installed 状态)。
#[tauri::command]
pub async fn list_databases<R: Runtime>(app: AppHandle<R>) -> Result<Vec<DatabaseInfo>, String> {
    let dir = drivers_dir(&app)?;
    Ok(state::list_databases_with_state(&dir))
}

/// 安装 JDBC 驱动(copy jar 到 drivers/ + 写 installed)。
#[tauri::command]
pub async fn install_driver<R: Runtime>(
    app: AppHandle<R>,
    dialect: String,
    jar_path: String,
) -> Result<(), String> {
    let dir = drivers_dir(&app)?;
    state::install_driver(&dir, &dialect, &jar_path)
}

/// 卸载 JDBC 驱动(删 jar + 删 installed)。
#[tauri::command]
pub async fn uninstall_driver<R: Runtime>(
    app: AppHandle<R>,
    dialect: String,
) -> Result<(), String> {
    let dir = drivers_dir(&app)?;
    state::uninstall_driver(&dir, &dialect)
}
