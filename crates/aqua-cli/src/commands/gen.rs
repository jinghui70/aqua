//! 生成命令:gen entity / gen datamodel —— 复用 aqua-core 生成器,产物打到 stdout。
//!
//! 产物放哪由消费项目的目录规范决定(只有 AI 知道),故 CLI 只输出、不落盘。
//! 包名由生成器从 .aqua 的 basePackage + 分组自算,不需传入。

use crate::load::load;
use anyhow::{anyhow, Result};
use aqua_core::generators::frontend_json::{generate_frontend_json, FrontendJsonOptions};
use aqua_core::generators::java::{generate_java_entity, JavaOptions};

/// 生成 dba 规范 entity Java。
pub fn entity(file: &str, table: &str) -> Result<()> {
    let project = load(file)?;
    let code = generate_java_entity(&project, table, &JavaOptions::default())
        .map_err(|e| anyhow!("生成 entity 失败: {e}"))?;
    print!("{code}");
    Ok(())
}

/// 生成 json-ui DataModel JSON。
pub fn datamodel(file: &str, table: &str) -> Result<()> {
    let project = load(file)?;
    // generate_frontend_json 在表不存在时会 panic,先校验给出友好错误。
    if !project.tables.iter().any(|t| t.code == table) {
        return Err(anyhow!("表不存在: {table}"));
    }
    let json = generate_frontend_json(
        &project,
        &FrontendJsonOptions {
            table: Some(table.to_string()),
        },
    );
    print!("{json}");
    Ok(())
}
