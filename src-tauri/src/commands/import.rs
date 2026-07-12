//! 导入 Tauri commands。

use aqua_core::driver::{create_driver, DbConfig};
use aqua_core::import::import_from_db;
use aqua_core::schema::Project;

/// 测试数据库连接。
#[tauri::command]
pub async fn test_connection_command(config: DbConfig) -> Result<String, String> {
    let driver = create_driver(config).map_err(|e| e.to_string())?;
    driver
        .test_connection()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    Ok("连接成功".to_string())
}

/// 从数据库导入 schema,返回 Project。
#[tauri::command]
pub async fn import_from_db_command(
    config: DbConfig,
    base_package: Option<String>,
) -> Result<Project, String> {
    let schema = config
        .schema
        .clone()
        .unwrap_or_else(|| config.database.clone());
    let driver = create_driver(config).map_err(|e| format!("创建驱动失败: {}", e))?;
    import_from_db(driver.as_ref(), &schema, base_package)
        .await
        .map_err(|e| format!("导入失败: {}", e))
}
