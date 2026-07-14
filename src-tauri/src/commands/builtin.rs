//! 内置业务类型清单加载 Tauri command。
//!
//! 清单为外置资源文件 `resources/builtin-biztypes.json`,随产物分发。

use aqua_core::schema::BizTypeDefine;
use serde::Deserialize;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Deserialize)]
struct BuiltinFile {
    #[serde(rename = "bizTypes")]
    biz_types: Vec<BizTypeDefine>,
}

/// 读取内置业务类型清单(打包资源文件)。
#[tauri::command]
pub async fn builtin_biztypes_load<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<BizTypeDefine>, String> {
    let path = app
        .path()
        .resolve("resources/builtin-biztypes.json", BaseDirectory::Resource)
        .map_err(|e| format!("定位内置清单失败: {}", e))?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取内置清单失败: {}", e))?;
    let file: BuiltinFile =
        serde_json::from_str(&content).map_err(|e| format!("内置清单 JSON 非法: {}", e))?;
    Ok(file.biz_types)
}
