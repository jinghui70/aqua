//! 项目管理 Tauri commands。

use aqua_core::schema::{validate_project, Project, ValidationError};
use aqua_core::version::{check_version_compatibility, VersionCheck, AQUA_VERSION};
use serde::{Deserialize, Serialize};

/// 项目打开结果,包含版本检查信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ProjectOpenResult {
    /// 成功打开,版本兼容
    Success { project: Project },
    /// 成功打开,文件为旧版本(会在保存时升级)
    CanOpen {
        project: Project,
        file_version: String,
        current_version: String,
    },
    /// 拒绝打开,文件版本过高需要升级 app
    NeedUpgrade {
        file_version: String,
        current_version: String,
    },
}

/// 读取并解析 schema.json 文件,包含版本兼容性检查。
#[tauri::command]
pub async fn project_open(path: String) -> Result<ProjectOpenResult, String> {
    let json_str = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取文件 {} 失败: {}", path, e))?;

    let value: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let project = Project::from_json(value).map_err(|e| format!("schema 结构错误: {}", e))?;

    // 版本兼容性检查
    let result = match check_version_compatibility(&project.version) {
        VersionCheck::Compatible => ProjectOpenResult::Success { project },
        VersionCheck::CanOpen { file_version } => ProjectOpenResult::CanOpen {
            project,
            file_version,
            current_version: AQUA_VERSION.to_string(),
        },
        VersionCheck::NeedUpgrade { file_version } => ProjectOpenResult::NeedUpgrade {
            file_version,
            current_version: AQUA_VERSION.to_string(),
        },
    };

    Ok(result)
}

/// 保存 Project 为 schema.json,自动将 version 升级为当前 app 版本。
#[tauri::command]
pub async fn project_save(path: String, mut project: Project) -> Result<(), String> {
    // 保存时自动升级版本为当前 app 版本
    project.version = AQUA_VERSION.to_string();

    let json =
        serde_json::to_string_pretty(&project).map_err(|e| format!("JSON 序列化失败: {}", e))?;

    tokio::fs::write(&path, json)
        .await
        .map_err(|e| format!("保存文件 {} 失败: {}", path, e))?;

    Ok(())
}

/// 校验 Project,返回错误列表(空 Vec = 合法)。
#[tauri::command]
pub async fn project_validate(project: Project) -> Result<Vec<ValidationError>, String> {
    match validate_project(&project) {
        Ok(()) => Ok(vec![]),
        Err(errors) => Ok(errors),
    }
}

/// 更新项目目录下的 .gitignore，确保包含 *.aqua.conf(数据源配置,不入 Git)
/// 数据集 .data 是 JSONL 文本,入 Git(不排除);原 *.aqua.db 已放弃 SQLite,移除
#[tauri::command]
pub async fn update_gitignore(project_path: String) -> Result<(), String> {
    use std::path::Path;

    let path = Path::new(&project_path);
    let dir = path.parent().ok_or("无效项目路径")?;
    let gitignore_path = dir.join(".gitignore");

    let patterns = vec!["*.aqua.conf"];

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
