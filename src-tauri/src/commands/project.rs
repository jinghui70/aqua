//! 项目管理 Tauri commands。

use aqua_core::schema::{parse_project, validate_project, ParseError, Project};

/// 读取并解析 schema.json 文件。
#[tauri::command]
pub async fn project_open(path: String) -> Result<Project, String> {
    let json_str = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取文件 {} 失败: {}", path, e))?;

    let value: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析失败: {}", e))?;

    parse_project(value).map_err(|e| match e {
        ParseError::Deserialize(err) => format!("schema 结构错误: {}", err),
        ParseError::Validate(errors) => serde_json::to_string(&errors)
            .unwrap_or_else(|_| format!("校验失败: {} 个错误", errors.len())),
    })
}

/// 保存 Project 为 schema.json。
#[tauri::command]
pub async fn project_save(path: String, project: Project) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(&project).map_err(|e| format!("JSON 序列化失败: {}", e))?;

    tokio::fs::write(&path, json)
        .await
        .map_err(|e| format!("保存文件 {} 失败: {}", path, e))?;

    Ok(())
}

/// 校验 Project,返回错误列表(JSON 字符串)。
#[tauri::command]
pub async fn project_validate(project: Project) -> Result<String, String> {
    match validate_project(&project) {
        Ok(()) => Ok("{}".to_string()),
        Err(errors) => {
            serde_json::to_string(&errors).map_err(|_| format!("校验失败: {} 个错误", errors.len()))
        }
    }
}

/// 更新项目目录下的 .gitignore，确保包含 *.aqua.conf 和 *.aqua.db
#[tauri::command]
pub async fn update_gitignore(project_path: String) -> Result<(), String> {
    use std::path::Path;

    let path = Path::new(&project_path);
    let dir = path.parent().ok_or("无效项目路径")?;
    let gitignore_path = dir.join(".gitignore");

    let patterns = vec!["*.aqua.conf", "*.aqua.db"];

    // 读取现有内容
    let mut content = if gitignore_path.exists() {
        tokio::fs::read_to_string(&gitignore_path)
            .await
            .map_err(|e| format!("读取 .gitignore 失败: {}", e))?
    } else {
        String::new()
    };

    // 检查并追加缺失的模式
    let mut modified = false;
    for pattern in patterns {
        if !content.lines().any(|line| line.trim() == pattern) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(pattern);
            content.push('\n');
            modified = true;
        }
    }

    // 仅在有变更时写入
    if modified || !gitignore_path.exists() {
        tokio::fs::write(&gitignore_path, content)
            .await
            .map_err(|e| format!("写入 .gitignore 失败: {}", e))?;
    }

    Ok(())
}
