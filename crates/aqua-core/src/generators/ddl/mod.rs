//! DDL 生成器 - 从 Project 生成 CREATE TABLE/INDEX DDL。
//!
//! 支持内置方言(MySQL/PostgreSQL) + 外置方言(通过 connector.jar)。
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/ddl/`。

mod index;
mod table;
pub mod types;

pub use types::{escape_sql_string, DdlOptions, Dialect};
// 供 alter 生成器复用
pub use index::generate_index;
pub use table::generate_table;

use crate::dataset::DatasetEntry;
use crate::schema::{Project, Table};
use serde_json::Value;

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

/// 生成 INSERT 语句(按 options.tables/group 过滤表,单行格式)。
/// 用于 DDL 导出追加数据集数据;空表跳过。
pub fn generate_insert(
    project: &Project,
    entries: &[DatasetEntry],
    options: &DdlOptions,
) -> String {
    let tables = filter_tables(project, options);
    let mut output = Vec::new();
    for table in tables {
        let Some(entry) = entries.iter().find(|e| e.table == table.code) else {
            continue;
        };
        if entry.data.is_empty() {
            continue;
        }
        let cols: Vec<&str> = table.fields.iter().map(|f| f.code.as_str()).collect();
        for row in &entry.data {
            let vals: Vec<String> = table
                .fields
                .iter()
                .map(|f| {
                    let key = f.code.to_uppercase();
                    format_literal(row.get(&key).unwrap_or(&Value::Null))
                })
                .collect();
            output.push(format!(
                "INSERT INTO {} ({}) VALUES ({});",
                table.code,
                cols.join(", "),
                vals.join(", ")
            ));
        }
    }
    if output.is_empty() {
        String::new()
    } else {
        output.join("\n") + "\n"
    }
}

/// SQL 值字面量: NULL -> NULL,数字原样,布尔 -> 1/0,字符串转义,其他 JSON 字符串。
fn format_literal(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "1".into()
            } else {
                "0".into()
            }
        }
        Value::String(s) => format!("'{}'", escape_sql_string(s)),
        other => format!("'{}'", escape_sql_string(&other.to_string())),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::DatasetEntry;
    use crate::schema::{DataType, Field, GroupDefine, Table};
    use serde_json::{json, Map};

    fn make_project() -> Project {
        Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            biz_types: vec![],
            auto_gen_strategies: vec![],
            groups: vec![GroupDefine {
                code: "default".to_string(),
                name: "默认".to_string(),
            }],
            tables: vec![Table {
                code: "SYS_USER".to_string(),
                name: "用户".to_string(),
                group: "default".to_string(),
                fields: vec![
                    Field {
                        prop: "id".into(),
                        code: "ID".into(),
                        name: "主键".into(),
                        data_type: DataType::Long,
                        length: None,
                        precision: None,
                        scale: None,
                        biz_type: None,
                        biz_type_data: None,
                        is_key: Some(true),
                        not_null: Some(true),
                        auto_generate: None,
                        default_value: None,
                        enum_ref: None,
                        comment: None,
                    },
                    Field {
                        prop: "userName".into(),
                        code: "USER_NAME".into(),
                        name: "用户名".into(),
                        data_type: DataType::Varchar,
                        length: Some(64),
                        precision: None,
                        scale: None,
                        biz_type: None,
                        biz_type_data: None,
                        is_key: None,
                        not_null: Some(true),
                        auto_generate: None,
                        default_value: None,
                        enum_ref: None,
                        comment: None,
                    },
                ],
                indexes: None,
                comment: None,
            }],
        }
    }

    #[test]
    fn test_generate_insert() {
        let project = make_project();
        let mut row1 = Map::new();
        row1.insert("ID".into(), json!(1));
        row1.insert("USER_NAME".into(), json!("admin"));
        let mut row2 = Map::new();
        row2.insert("ID".into(), json!(2));
        row2.insert("USER_NAME".into(), Value::Null);
        let entries = vec![DatasetEntry {
            table: "SYS_USER".into(),
            data: vec![row1, row2],
        }];
        let sql = generate_insert(
            &project,
            &entries,
            &DdlOptions {
                dialect: Dialect::Mysql,
                tables: Some(vec!["SYS_USER".into()]),
                group: None,
                drop_if_exist: true,
            },
        );
        assert!(sql.contains("INSERT INTO SYS_USER (ID, USER_NAME) VALUES (1, 'admin');"));
        assert!(sql.contains("INSERT INTO SYS_USER (ID, USER_NAME) VALUES (2, NULL);"));
    }

    #[test]
    fn test_generate_insert_filters_tables_and_skips_empty() {
        let project = make_project();
        // 数据集为空 -> 无 INSERT
        let entries = vec![DatasetEntry {
            table: "SYS_USER".into(),
            data: vec![],
        }];
        let sql = generate_insert(
            &project,
            &entries,
            &DdlOptions::default(),
        );
        assert!(sql.is_empty());
        // 选中不存在的表 -> 无 INSERT
        let sql2 = generate_insert(
            &project,
            &entries,
            &DdlOptions {
                tables: Some(vec!["NOT_EXIST".into()]),
                ..Default::default()
            },
        );
        assert!(sql2.is_empty());
    }

    #[test]
    fn test_format_literal() {
        assert_eq!(format_literal(&Value::Null), "NULL");
        assert_eq!(format_literal(&json!(42)), "42");
        assert_eq!(format_literal(&json!(true)), "1");
        assert_eq!(format_literal(&json!(false)), "0");
        assert_eq!(format_literal(&json!("a'b")), "'a''b'");
    }
}
