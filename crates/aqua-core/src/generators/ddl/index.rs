//! CREATE INDEX 生成逻辑。

use crate::schema::{Index, Table};

/// 生成 CREATE INDEX 语句: `CREATE [UNIQUE] INDEX name ON table (fields);`。
pub fn generate_index(table: &Table, index: &Index) -> String {
    let table_name = table.code.to_uppercase();
    let fields = index
        .fields
        .iter()
        .map(|f| f.to_uppercase())
        .collect::<Vec<_>>()
        .join(", ");

    let unique = if index.unique { "UNIQUE " } else { "" };

    let name = index
        .name
        .as_ref()
        .map(|n| n.to_uppercase())
        .unwrap_or_else(|| auto_index_name(&table.code, &index.fields));

    format!(
        "CREATE {}INDEX {} ON {} ({});",
        unique, name, table_name, fields
    )
}

/// 自动索引名: `IDX_<TABLE>_<F1>_<F2>`。
fn auto_index_name(table: &str, fields: &[String]) -> String {
    format!("IDX_{}_{}", table, fields.join("_")).to_uppercase()
}
