//! Java 实体生成器 - 按 rainbow-dbaccess 规范生成实体类。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/java-entity/`。

pub mod entity;
pub mod enum_class;
pub mod naming;
pub mod types;

pub use types::JavaOptions;

use crate::schema::Project;

/// Java 实体生成入口(单表)。
pub fn generate_java_entity(
    project: &Project,
    table_code: &str,
    options: &JavaOptions,
) -> Result<String, String> {
    let table = project
        .tables
        .iter()
        .find(|t| t.code == table_code)
        .ok_or_else(|| format!("Table not found: {}", table_code))?;

    entity::generate_entity_class(project, table, options)
}
