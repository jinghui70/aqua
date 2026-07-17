//! 导入 Tauri commands。

use aqua_core::driver::{create_driver, DbConfig};
use aqua_core::import::import_from_db;
use aqua_core::schema::Project;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

use super::database::drivers_dir;

/// 定位 bundle 内 connector.jar 绝对路径(dev/打包一致)。
///
/// connector.jar 作为 Tauri resource 打包(`bundle.resources`),走 resource_dir(只读);
/// 与外置 JDBC 驱动目录 `app_data_dir/drivers/`(用户可写)分离,不可混用。
/// 解析失败时回退相对路径 "connector.jar"(保旧行为,native 方言不受影响)。
fn connector_jar_path<R: Runtime>(app: &AppHandle<R>) -> String {
    app.path()
        .resolve("resources/connector.jar", BaseDirectory::Resource)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "connector.jar".to_string())
}

/// 测试数据库连接。
#[tauri::command]
pub async fn test_connection_command<R: Runtime>(
    app: AppHandle<R>,
    config: DbConfig,
) -> Result<String, String> {
    let drivers = drivers_dir(&app).ok();
    let connector = connector_jar_path(&app);
    let driver = create_driver(config, drivers, &connector).map_err(|e| e.to_string())?;
    driver
        .test_connection()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    Ok("连接成功".to_string())
}

/// 从数据库导入 schema,返回 Project。
#[tauri::command]
pub async fn import_from_db_command<R: Runtime>(
    app: AppHandle<R>,
    config: DbConfig,
    tables: Vec<String>,
    base_package: Option<String>,
) -> Result<Project, String> {
    let drivers = drivers_dir(&app).ok();
    let connector = connector_jar_path(&app);
    let driver = create_driver(config, drivers, &connector).map_err(|e| format!("创建驱动失败: {}", e))?;
    import_from_db(driver.as_ref(), &tables, base_package)
        .await
        .map_err(|e| format!("导入失败: {}", e))
}

/// 列出数据库所有表名(导入向导 Step2 选表用)。
#[tauri::command]
pub async fn list_tables_command<R: Runtime>(
    app: AppHandle<R>,
    config: DbConfig,
) -> Result<Vec<String>, String> {
    let schema = config
        .schema
        .clone()
        .unwrap_or_else(|| config.database.clone());
    let drivers = drivers_dir(&app).ok();
    let connector = connector_jar_path(&app);
    let driver = create_driver(config, drivers, &connector).map_err(|e| format!("创建驱动失败: {}", e))?;
    driver
        .list_tables(&schema)
        .await
        .map_err(|e| format!("列表失败: {}", e))
}
