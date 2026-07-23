//! DDL 生成器 - 从 Project 生成 CREATE TABLE/INDEX DDL。
//!
//! 支持内置方言(MySQL/PostgreSQL) + 外置方言(通过 connector.jar)。
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/ddl/`。

mod index;
mod table;
pub mod types;

pub use types::{DdlOptions, Dialect};
// 供 alter 生成器复用
pub use index::generate_index;
pub use table::generate_table;

use crate::schema::{Project, Table};

/// DDL 生成入口。输入已校验的 Project,输出可直接执行的 DDL 文本。
pub fn generate_ddl(project: &Project, options: &DdlOptions) -> String {
    let dialect = &options.dialect;
    let tables = filter_tables(project, options);

    let mut output = Vec::new();

    for table in tables {
        if options.drop_if_exist {
            let is_oracle = matches!(options.dialect, Dialect::Jdbc { ref name } if name == "oracle");
            if is_oracle {
                output.push(format!(
                    "BEGIN\n   EXECUTE IMMEDIATE 'DROP TABLE {} PURGE';\nEXCEPTION\n   WHEN OTHERS THEN\n      IF SQLCODE != -942 THEN\n         RAISE;\n      END IF;\nEND;",
                    table.code
                ));
            } else {
                output.push(format!("DROP TABLE IF EXISTS {};", table.code));
            }
        }
        // CREATE TABLE + COMMENT
        output.push(table::generate_table(table, dialect));

        // CREATE INDEX
        if let Some(ref indexes) = table.indexes {
            for idx in indexes {
                output.push(index::generate_index(table, idx));
            }
        }
    }

    if output.is_empty() {
        String::new()
    } else {
        output.join("\n\n") + "\n"
    }
}

/// 按 options 过滤表: tables(表名列表) 或 group(分组) 或全部表。
fn filter_tables<'a>(project: &'a Project, options: &DdlOptions) -> Vec<&'a Table> {
    use std::collections::HashSet;

    if let Some(ref table_codes) = options.tables {
        let set: HashSet<_> = table_codes.iter().map(|s| s.as_str()).collect();
        project
            .tables
            .iter()
            .filter(|t| set.contains(t.code.as_str()))
            .collect()
    } else if let Some(ref group) = options.group {
        project
            .tables
            .iter()
            .filter(|t| t.group == *group)
            .collect()
    } else {
        project.tables.iter().collect()
    }
}
