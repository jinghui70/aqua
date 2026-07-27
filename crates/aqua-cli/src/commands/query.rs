//! 查询命令:groups / tables / show -- 省 token 地读结构。

use crate::load::load;
use anyhow::{anyhow, Result};

/// 列出所有表组。
pub fn groups(file: &str) -> Result<()> {
    let project = load(file)?;
    for g in &project.groups {
        println!("{}\t{}", g.code, g.name);
    }
    Ok(())
}

/// 列出表(可按组过滤)。
pub fn tables(file: &str, group: Option<&str>) -> Result<()> {
    let project = load(file)?;
    for t in &project.tables {
        if let Some(g) = group {
            if t.group != g {
                continue;
            }
        }
        println!("{}\t{}\t({})", t.code, t.name, t.group);
    }
    Ok(())
}

/// 显示单表结构(JSON)。
///
/// 清理:仅 VARCHAR 显示 length,仅 DECIMAL 显示 precision/scale;
/// 其余类型(TINYINT/INT/LONG/DATE/DATETIME/CLOB/BLOB)去掉无意义的 length。
/// None 字段(bizType 等)由 serde skip_serializing_if 自动省略。
pub fn show(file: &str, table_code: &str) -> Result<()> {
    let project = load(file)?;
    let table = project
        .tables
        .iter()
        .find(|t| t.code == table_code)
        .ok_or_else(|| anyhow!("表不存在: {table_code}"))?;
    let mut value = serde_json::to_value(table)?;
    if let Some(fields) = value.get_mut("fields").and_then(|f| f.as_array_mut()) {
        for field in fields {
            let dt = field
                .get("dataType")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            if dt != "VARCHAR" {
                if let Some(obj) = field.as_object_mut() {
                    obj.remove("length");
                }
            }
            if dt != "DECIMAL" {
                if let Some(obj) = field.as_object_mut() {
                    obj.remove("precision");
                    obj.remove("scale");
                }
            }
            // defaultValue: 数字类型转 JSON number(存储是字符串,显示应类型正确)
            if matches!(dt.as_str(), "TINYINT" | "INT" | "LONG" | "DECIMAL") {
                if let Some(obj) = field.as_object_mut() {
                    let default_val = obj
                        .get("defaultValue")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned());
                    if let Some(default) = default_val {
                        if let Ok(n) = default.parse::<i64>() {
                            obj.insert("defaultValue".to_string(), serde_json::Value::from(n));
                        } else if let Ok(n) = default.parse::<f64>() {
                            obj.insert("defaultValue".to_string(), serde_json::Value::from(n));
                        }
                    }
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
