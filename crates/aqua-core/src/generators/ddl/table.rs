//! CREATE TABLE 生成逻辑。

use super::types::{escape_sql_string, map_type, Dialect};
use crate::schema::{Field, Table};

/// 生成 CREATE TABLE + COMMENT 语句。
pub fn generate_table(table: &Table, dialect: &Dialect) -> String {
    let table_name = table.code.to_uppercase();

    // 字段定义 + 主键,统一用 ",\n" 连接(避免末尾逗号或缺逗号)
    let mut defs: Vec<String> = table
        .fields
        .iter()
        .map(|f| format!("  {}", field_definition(f, dialect)))
        .collect();

    if let Some(pk) = primary_key_clause(table) {
        defs.push(pk);
    }

    let body = defs.join(",\n");

    let mut output = if inline_table_comment(dialect) {
        // MySQL: CREATE TABLE (...) COMMENT 'xxx';
        format!(
            "CREATE TABLE {} (\n{}\n) COMMENT '{}';",
            table_name,
            body,
            escape_sql_string(&table.name)
        )
    } else {
        format!("CREATE TABLE {} (\n{}\n);", table_name, body)
    };

    // 独立表注释 (PG/Oracle/KingBase 等;SQL Server 不支持)
    if !inline_table_comment(dialect) && supports_comment(dialect) {
        output.push_str(&format!(
            "\nCOMMENT ON TABLE {} IS '{}';",
            table_name,
            escape_sql_string(&table.name)
        ));
    }

    // 独立列注释 (PG/Oracle/KingBase 等;SQL Server 不支持)
    if !inline_column_comment(dialect) && supports_comment(dialect) {
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
        // MySQL 兼容系 + DM/GBase/H2 支持内联列注释
        Dialect::Jdbc { name } => {
            matches!(name.as_str(), "h2" | "dm" | "gbase" | "oceanbase" | "tidb")
        }
    }
}

/// 方言是否支持内联表注释(CREATE TABLE 后 COMMENT)。
fn inline_table_comment(dialect: &Dialect) -> bool {
    match dialect {
        Dialect::Mysql => true,
        Dialect::Postgresql => false,
        // MySQL 兼容系支持内联表注释
        Dialect::Jdbc { name } => matches!(name.as_str(), "oceanbase" | "tidb"),
    }
}

/// 方言是否支持注释(SQL Server 用 sp_addextendedproperty,c1 暂不生成)。
fn supports_comment(dialect: &Dialect) -> bool {
    if let Dialect::Jdbc { name } = dialect {
        name != "sqlserver"
    } else {
        true
    }
}
