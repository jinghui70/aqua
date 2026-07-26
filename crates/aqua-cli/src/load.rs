//! 读取并解析 .aqua 文件为已校验的 Project。
//!
//! 复用 aqua-core 的 `parse_project`(含反序列化 + 业务校验),CLI 不另写校验。

use anyhow::{anyhow, Context, Result};
use aqua_core::schema::{parse_project, ParseError, Project};

/// 读文件 → JSON Value → parse_project(校验)。错误分类为可读文本。
pub fn load(path: &str) -> Result<Project> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("读取文件失败: {path}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&content).with_context(|| format!("JSON 解析失败: {path}"))?;
    parse_project(value).map_err(|e| match e {
        ParseError::Deserialize(err) => anyhow!("schema 结构错误: {err}"),
        ParseError::Validate(errors) => {
            let mut msg = format!("schema 校验失败（{} 个错误）：", errors.len());
            for err in &errors {
                msg.push_str(&format!("\n  - {err:?}"));
            }
            anyhow!(msg)
        }
    })
}
