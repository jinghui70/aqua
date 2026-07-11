//! CREATE TABLE 生成逻辑。

use super::types::{escape_sql_string, map_type, Dialect};
use crate::schema::{Field, Table};

/// 生成 CREATE TABLE + COMMENT 语句。
pub fn generate_table(table: &Table, dialect: &Dialect) -> String {
    let table_name = table.code.to_uppercase();
    let mut lines = vec![format!("CREATE TABLE {} (", table_name)];

    // 字段定义
    for (i, field) in table.fields.iter().enumerate() {
        let comma = if i < table.fields.len() - 1 { "," } else { "" };
        lines.push(format!("  {}{}", field_definition(field, dialect), comma));
    }

    // PRIMARY KEY
    if let Some(pk) = primary_key_clause(table) {
        lines.push(pk);
    }

    // 表结束
    if inline_table_comment(dialect) {
        // MySQL: CREATE TABLE (...) COMMENT 'xxx';
        lines.push(format!(") COMMENT '{}';", escape_sql_string(&table.name)));
    } else {
        lines.push(");".to_string());
    }

    let mut output = lines.join("\n");

    // 独立表注释 (PG/Oracle/KingBase)
    if !inline_table_comment(dialect) {
        output.push_str(&format!(
            "\nCOMMENT ON TABLE {} IS '{}';",
            table_name,
            escape_sql_string(&table.name)
        ));
    }

    // 独立列注释 (PG/Oracle/KingBase)
    if !inline_column_comment(dialect) {
        for field in &table.fields {
            output.push_str(&format!(
                "\nCOMMENT ON COLUMN {}.{} IS '{}';",
                table_name,
                field.code.to_uppercase(),
                escape_sql_string(&field.name)
            ));
        }
    }

    output
}

/// 字段定义: `CODE TYPE [NOT NULL] [DEFAULT xxx] [COMMENT 'xxx']`。
fn field_definition(field: &Field, dialect: &Dialect) -> String {
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

    let default = if let Some(ref val) = field.default_value {
        format!(" DEFAULT {}", val) // 已转义,直接用
    } else {
        String::new()
    };

    let comment = if inline_column_comment(dialect) {
        format!(" COMMENT '{}'", escape_sql_string(&field.name))
    } else {
        String::new()
    };

    format!("{} {}{}{}{}", name, data_type, not_null, default, comment)
}

/// PRIMARY KEY 子句: `PRIMARY KEY (F1, F2)`。
fn primary_key_clause(table: &Table) -> Option<String> {
    let keys: Vec<_> = table
        .fields
        .iter()
        .filter(|f| f.is_key.unwrap_or(false))
        .map(|f| f.code.to_uppercase())
        .collect();

    if keys.is_empty() {
        None
    } else {
        Some(format!("  PRIMARY KEY ({})", keys.join(", ")))
    }
}

/// 方言是否支持内联列注释(字段定义内 COMMENT)。
fn inline_column_comment(dialect: &Dialect) -> bool {
    match dialect {
        Dialect::Mysql => true,
        Dialect::Postgresql => false,
        Dialect::Jdbc { name } => matches!(name.as_str(), "h2" | "dm" | "gbase"),
    }
}

/// 方言是否支持内联表注释(CREATE TABLE 后 COMMENT)。
fn inline_table_comment(dialect: &Dialect) -> bool {
    matches!(dialect, Dialect::Mysql)
}
