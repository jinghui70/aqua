//! Java 实体生成器 - 按 rainbow-dbaccess 规范生成实体类。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/java-entity/`。
//! 枚举字段:生成独立枚举类文件(`JavaFile[]`),实体字段类型引用枚举类名。

pub mod entity;
pub mod enumeration;
pub mod naming;
pub mod types;

pub use types::JavaOptions;

use crate::schema::Project;
use serde::{Deserialize, Serialize};

/// 生成产物:一个 Java 文件(path 为相对路径,content 为文件内容)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaFile {
    pub path: String,
    pub content: String,
}

/// Java 实体生成入口(单表)。返回实体文件 + 本表定义方的枚举文件(Vec 有序)。
pub fn generate_java_entity(
    project: &Project,
    table_code: &str,
    options: &JavaOptions,
) -> Result<Vec<JavaFile>, String> {
    let table = project
        .tables
        .iter()
        .find(|t| t.code == table_code)
        .ok_or_else(|| format!("Table not found: {}", table_code))?;

    entity::generate_entity_files(project, table, options)
}
