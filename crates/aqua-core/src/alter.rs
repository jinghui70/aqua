//! ALTER DDL 生成器 - 基于 diff 结果生成 ALTER 语句。
//!
//! 规则见 `docs/design.md` §4.3。
//! 依赖: diff 引擎 + DDL 生成器(复用 CREATE TABLE/INDEX)。

use crate::diff::{DiffResult, TableChange};
use crate::generators::ddl::types::{escape_sql_string, map_type, Dialect};
use crate::generators::ddl::{generate_index, generate_table};
use crate::schema::{Field, Index, Project, Table};

/// ALTER 生成选项。
#[derive(Debug, Clone)]
pub struct AlterOptions {
    pub dialect: Dialect,
}

impl Default for AlterOptions {
    fn default() -> Self {
        Self {
            dialect: Dialect::default(),
        }
    }
}

/// 从 diff 结果生成 ALTER DDL。
///
/// `new_project`: 新版 Project(用于新增表/字段的完整定义)。
pub fn generate_alter(diff: &DiffResult, new_project: &Project, options: &AlterOptions) -> String {
    let dialect = &options.dialect;
    let mut sections: Vec<String> = Vec::new();

    // 1. 新增表(复用 ddl::generate_table)
    for table_code in &diff.tables.added {
        let table = new_project.tables.iter().find(|t| t.code == *table_code);
        if let Some(t) = table {
            sections.push(format!(
                "-- 新增表: {}\n{}",
                table_code,
                generate_table(t, dialect)
            ));
        }
    }

    // 2. 删除表
    for table_code in &diff.tables.removed {
        sections.push(format!(
            "-- 删除表: {}\nDROP TABLE {};",
            table_code,
            table_code.to_uppercase()
        ));
    }

    // 3. 表变更(字段/索引)
    for change in &diff.tables.changed {
        let table = new_project.tables.iter().find(|t| t.code == change.table);
        if let Some(t) = table {
            let alter = generate_table_alter(change, t, dialect);
            if !alter.is_empty() {
                sections.push(format!("-- 变更表: {}\n{}", change.table, alter));
            }
        }
    }

    if sections.is_empty() {
        String::new()
    } else {
        sections.join("\n\n") + "\n"
    }
}

/// 生成单表的 ALTER 语句(字段 + 索引)。
fn generate_table_alter(change: &TableChange, table: &Table, dialect: &Dialect) -> String {
    let table_name = table.code.to_uppercase();
    let mut stmts: Vec<String> = Vec::new();

    // 新增字段
    for field_code in &change.fields.added {
        if let Some(field) = table.fields.iter().find(|f| f.code == *field_code) {
            stmts.push(format!(
                "ALTER TABLE {} ADD COLUMN {};",
                table_name,
                column_def(field, dialect)
            ));
        }
    }

    // 删除字段
    for field_code in &change.fields.removed {
        stmts.push(format!(
            "ALTER TABLE {} DROP COLUMN {};",
            table_name,
            field_code.to_uppercase()
        ));
    }

    // 修改字段
    for fc in &change.fields.changed {
        if let Some(field) = table.fields.iter().find(|f| f.code == fc.field) {
            stmts.push(modify_column_stmt(&table_name, field, dialect));
        }
    }

    // 新增索引(复用 ddl::generate_index)
    for idx_key in &change.indexes.added {
        if let Some(idx) = find_index_by_key(table.indexes.as_ref(), idx_key) {
            stmts.push(generate_index(table, idx));
        }
    }

    // 删除索引
    for idx_key in &change.indexes.removed {
        let name = idx_key.split("IDX_").next().unwrap_or(idx_key);
        stmts.push(format!("DROP INDEX {};", name.to_uppercase()));
    }

    stmts.join("\n")
}

/// 字段定义(code type [NOT NULL] [DEFAULT])。
fn column_def(field: &Field, dialect: &Dialect) -> String {
    let name = field.code.to_uppercase();
    let data_type = map_type(
        field.data_type,
        field.length,
        field.precision,
        field.scale,
        dialect,
    );
    let not_null = if field.not_null.unwrap_or(false) {
        " NOT NULL"
    } else {
        ""
    };
    let default = if let Some(ref v) = field.default_value {
        format!(" DEFAULT {}", v)
    } else {
        String::new()
    };
    format!("{} {}{}{}", name, data_type, not_null, default)
}

/// 修改字段语句(方言相关)。
fn modify_column_stmt(table_name: &str, field: &Field, dialect: &Dialect) -> String {
    let col = field.code.to_uppercase();
    let def = column_def(field, dialect);

    match dialect {
        Dialect::Mysql => format!("ALTER TABLE {} MODIFY COLUMN {};", table_name, def),
        Dialect::Postgresql => {
            // PG: ALTER COLUMN ... TYPE(仅类型,简化处理)
            let data_type = map_type(
                field.data_type,
                field.length,
                field.precision,
                field.scale,
                dialect,
            );
            format!(
                "ALTER TABLE {} ALTER COLUMN {} TYPE {};",
                table_name, col, data_type
            )
        }
        Dialect::Jdbc { name } => match name.as_str() {
            "oracle" => format!("ALTER TABLE {} MODIFY ({});", table_name, def),
            _ => format!(
                "ALTER TABLE {} ALTER COLUMN {} {};",
                table_name,
                col,
                def_type(field, dialect)
            ),
        },
    }
}

/// 字段类型部分(用于 H2 等)。
fn def_type(field: &Field, dialect: &Dialect) -> String {
    let data_type = map_type(
        field.data_type,
        field.length,
        field.precision,
        field.scale,
        dialect,
    );
    let not_null = if field.not_null.unwrap_or(false) {
        " NOT NULL"
    } else {
        ""
    };
    format!("{}{}", data_type, not_null)
}

/// 按 diff 索引 key 查找索引。
fn find_index_by_key<'a>(indexes: Option<&'a Vec<Index>>, key: &str) -> Option<&'a Index> {
    let indexes = indexes?;
    indexes.iter().find(|idx| {
        let k = idx
            .name
            .clone()
            .unwrap_or_else(|| format!("IDX_{}", idx.fields.join("_")));
        k == key
    })
}

// 避免未使用警告(escape_sql_string 在 column_def 暂未用,但保留供扩展)
#[allow(dead_code)]
fn _unused() {
    let _ = escape_sql_string("");
}
