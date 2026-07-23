//! dataset 模块 - JSONL 数据集(.data)。
//!
//! 每行 `{"table":"SYS_USER","row":{"ID":1,"NAME":"admin"}}`,不存表结构(结构用主项目)。
//! 保存时按第一个主键字段值排序,保证文件稳定 diff。

use crate::schema::{Project, Table};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use thiserror::Error;

/// dataset 错误类型。
#[derive(Error, Debug)]
pub enum DatasetError {
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("表不存在: {0}")]
    TableNotFound(String),
    #[error("数据集与项目表结构不一致: {0}")]
    SchemaMismatch(String),
}

/// JSONL 行:`{"table":"SYS_USER","row":{...}}`
#[derive(Serialize, Deserialize)]
struct JsonlRow {
    table: String,
    row: Map<String, Value>,
}

/// 数据集条目(表 -> 行数据)。前端/命令层用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub table: String,
    pub data: Vec<Map<String, Value>>,
}

/// 加载数据集(.data JSONL)-> 按表分组 -> DatasetEntry 列表(按项目表顺序)。
pub fn load_dataset(path: &str, project: &Project) -> Result<Vec<DatasetEntry>, DatasetError> {
    let content = std::fs::read_to_string(path)?;
    let mut map: BTreeMap<String, Vec<Map<String, Value>>> = BTreeMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: JsonlRow = serde_json::from_str(line)?;
        map.entry(row.table).or_default().push(row.row);
    }
    // 按项目表顺序输出(项目有但数据集无 -> 空数据)
    let entries: Vec<DatasetEntry> = project
        .tables
        .iter()
        .map(|t| DatasetEntry {
            table: t.code.clone(),
            data: map.remove(&t.code).unwrap_or_default(),
        })
        .collect();
    validate_against(project, &entries)?;
    Ok(entries)
}

/// 保存数据集(.data JSONL,按主键排序)。
pub fn save_dataset(
    path: &str,
    project: &Project,
    entries: &[DatasetEntry],
) -> Result<(), DatasetError> {
    validate_against(project, entries)?;
    let mut lines = Vec::new();
    for entry in entries {
        let table = project.tables.iter().find(|t| t.code == entry.table);
        let sorted = sort_rows_by_pk(table, &entry.data);
        for row in &sorted {
            let line = serde_json::to_string(&JsonlRow {
                table: entry.table.clone(),
                row: row.clone(),
            })?;
            lines.push(line);
        }
    }
    std::fs::write(path, lines.join("\n") + "\n")?;
    Ok(())
}

/// 按第一个主键字段值排序(数字序/字符串序),无主键保持原序。
fn sort_rows_by_pk(table: Option<&Table>, rows: &[Map<String, Value>]) -> Vec<Map<String, Value>> {
    let pk_code = match table.and_then(|t| t.fields.iter().find(|f| f.is_key.unwrap_or(false))) {
        Some(f) => f.code.to_uppercase(),
        None => return rows.to_vec(),
    };
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| compare_values(a.get(&pk_code).unwrap_or(&Value::Null), b.get(&pk_code).unwrap_or(&Value::Null)));
    sorted
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => {
            na.as_f64().unwrap_or(0.0).partial_cmp(&nb.as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
        (Value::Null, _) => std::cmp::Ordering::Less,
        (_, Value::Null) => std::cmp::Ordering::Greater,
        _ => a.to_string().cmp(&b.to_string()),
    }
}

/// 校验数据集条目与项目表结构一致(表存在、行 key 是合法字段 code)。
pub fn validate_against(project: &Project, entries: &[DatasetEntry]) -> Result<(), DatasetError> {
    for entry in entries {
        let table = project
            .tables
            .iter()
            .find(|t| t.code == entry.table)
            .ok_or_else(|| DatasetError::TableNotFound(entry.table.clone()))?;
        let valid: std::collections::HashSet<String> =
            table.fields.iter().map(|f| f.code.to_uppercase()).collect();
        for row in &entry.data {
            for k in row.keys() {
                if !valid.contains(&k.to_uppercase()) {
                    return Err(DatasetError::SchemaMismatch(format!(
                        "表 {} 无字段 {}",
                        entry.table, k
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{DataType, Field, GroupDefine, Table};

    fn make_project() -> Project {
        Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            biz_types: vec![],
            auto_gen_strategies: vec![],
            groups: vec![GroupDefine { code: "default".to_string(), name: "默认".to_string() }],
            tables: vec![Table {
                code: "SYS_USER".to_string(),
                name: "用户".to_string(),
                group: "default".to_string(),
                fields: vec![
                    Field { prop: "id".into(), code: "ID".into(), name: "主键".into(), data_type: DataType::Long, length: None, precision: None, scale: None, biz_type: None, biz_type_data: None, is_key: Some(true), not_null: Some(true), auto_generate: None, default_value: None, enum_ref: None, comment: None },
                    Field { prop: "userName".into(), code: "USER_NAME".into(), name: "用户名".into(), data_type: DataType::Varchar, length: Some(64), precision: None, scale: None, biz_type: None, biz_type_data: None, is_key: None, not_null: Some(true), auto_generate: None, default_value: None, enum_ref: None, comment: None },
                    Field { prop: "amount".into(), code: "AMOUNT".into(), name: "金额".into(), data_type: DataType::Decimal, length: None, precision: Some(12), scale: Some(2), biz_type: None, biz_type_data: None, is_key: None, not_null: None, auto_generate: None, default_value: None, enum_ref: None, comment: None },
                ],
                indexes: None,
                comment: None,
            }],
        }
    }

    fn sample_entries() -> Vec<DatasetEntry> {
        let mut row = Map::new();
        row.insert("ID".into(), Value::Number(2.into()));
        row.insert("USER_NAME".into(), Value::String("user".into()));
        row.insert("AMOUNT".into(), Value::String("1234567890.12".into()));
        let mut row2 = Map::new();
        row2.insert("ID".into(), Value::Number(1.into()));
        row2.insert("USER_NAME".into(), Value::Null);
        row2.insert("AMOUNT".into(), Value::String("99.50".into()));
        vec![DatasetEntry { table: "SYS_USER".into(), data: vec![row, row2] }]
    }

    #[test]
    fn test_jsonl_roundtrip() {
        let project = make_project();
        let entries = sample_entries();
        let tmp = "/tmp/aqua_test_dataset.data";
        let _ = std::fs::remove_file(tmp);

        save_dataset(tmp, &project, &entries).expect("保存失败");
        let loaded = load_dataset(tmp, &project).expect("加载失败");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].data.len(), 2);
        // 按主键排序:ID=1 在前
        assert_eq!(loaded[0].data[0]["ID"], Value::Number(1.into()));
        assert_eq!(loaded[0].data[1]["ID"], Value::Number(2.into()));
        // DECIMAL 保精度(字符串)
        assert_eq!(loaded[0].data[0]["AMOUNT"], Value::String("99.50".into()));
        // null 保留
        assert_eq!(loaded[0].data[0]["USER_NAME"], Value::Null);
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn test_validate_schema_mismatch() {
        let project = make_project();
        let bad_table = vec![DatasetEntry { table: "NOT_EXIST".into(), data: vec![] }];
        assert!(matches!(validate_against(&project, &bad_table), Err(DatasetError::TableNotFound(_))));

        let mut row = Map::new();
        row.insert("BAD_FIELD".into(), Value::Null);
        let bad_field = vec![DatasetEntry { table: "SYS_USER".into(), data: vec![row] }];
        assert!(matches!(validate_against(&project, &bad_field), Err(DatasetError::SchemaMismatch(_))));
    }

    #[test]
    fn test_empty_dataset() {
        let project = make_project();
        let tmp = "/tmp/aqua_test_empty.data";
        let _ = std::fs::remove_file(tmp);
        std::fs::write(tmp, "").unwrap();

        let loaded = load_dataset(tmp, &project).expect("加载空数据集失败");
        assert_eq!(loaded.len(), 1); // 项目有1张表
        assert!(loaded[0].data.is_empty()); // 无数据
        let _ = std::fs::remove_file(tmp);
    }
}
