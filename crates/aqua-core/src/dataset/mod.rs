//! dataset 模块 - SQLite 数据集容器。
//!
//! 规则见 `docs/design.md` §4.5。
//! .aqua 文件 = SQLite,内含 _aqua_meta(schema.json) + 数据表。

use crate::schema::{DataType, Project, Table};
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// dataset 错误类型。
#[derive(Error, Debug)]
pub enum DatasetError {
    #[error("SQLite 错误: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("表不存在: {0}")]
    TableNotFound(String),
}

/// SQLite 数据集容器。
pub struct Dataset {
    conn: Connection,
}

/// SQLite 类型映射(§4.5 保持精度:DECIMAL 用 TEXT)。
pub fn sqlite_type(data_type: DataType) -> &'static str {
    match data_type {
        DataType::Varchar | DataType::Clob => "TEXT",
        DataType::Tinyint | DataType::Int | DataType::Long => "INTEGER",
        DataType::Decimal => "TEXT", // 字符串避免精度丢失
        DataType::Date | DataType::Datetime => "TEXT",
        DataType::Blob => "BLOB",
    }
}

impl Dataset {
    /// 创建内存数据集,建 _aqua_meta + 数据表。
    pub fn new(project: &Project) -> Result<Self, DatasetError> {
        let conn = Connection::open_in_memory()?;

        // 元数据表
        conn.execute(
            "CREATE TABLE _aqua_meta (key TEXT PRIMARY KEY, value TEXT)",
            [],
        )?;

        // 存 schema.json
        let schema_json = serde_json::to_string(project)?;
        conn.execute(
            "INSERT INTO _aqua_meta (key, value) VALUES ('schema', ?1)",
            [&schema_json],
        )?;

        // 建数据表
        for table in &project.tables {
            let ddl = create_table_sql(table);
            conn.execute(&ddl, [])?;
        }

        Ok(Self { conn })
    }

    /// 从文件加载数据集。
    pub fn load(path: &str) -> Result<Self, DatasetError> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// 保存数据集到文件(用 backup API)。
    pub fn save(&self, path: &str) -> Result<(), DatasetError> {
        let mut dest = Connection::open(path)?;
        let backup = rusqlite::backup::Backup::new(&self.conn, &mut dest)?;
        backup.step(-1)?; // 全部复制
        Ok(())
    }

    /// 读取 schema.json 为 Project。
    pub fn get_project(&self) -> Result<Project, DatasetError> {
        let json: String = self.conn.query_row(
            "SELECT value FROM _aqua_meta WHERE key = 'schema'",
            [],
            |row| row.get(0),
        )?;
        let project: Project = serde_json::from_str(&json)?;
        Ok(project)
    }

    /// 执行原始 SQL(数据导入/查询用)。
    pub fn execute(&self, sql: &str) -> SqlResult<usize> {
        self.conn.execute(sql, [])
    }

    /// 获取连接引用(高级操作)。
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// 生成 SQLite CREATE TABLE 语句。
fn create_table_sql(table: &Table) -> String {
    let table_name = table.code.to_uppercase();

    let mut defs: Vec<String> = table
        .fields
        .iter()
        .map(|f| format!("  {} {}", f.code.to_uppercase(), sqlite_type(f.data_type)))
        .collect();

    // 主键
    let pk: Vec<String> = table
        .fields
        .iter()
        .filter(|f| f.is_key.unwrap_or(false))
        .map(|f| f.code.to_uppercase())
        .collect();
    if !pk.is_empty() {
        defs.push(format!("  PRIMARY KEY ({})", pk.join(", ")));
    }

    format!("CREATE TABLE {} (\n{}\n);", table_name, defs.join(",\n"))
}

/// 数据集 JSON 条目(§4.5 JSON 格式)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub table: String,
    pub data: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_project() -> Project {
        use crate::schema::{Field, GroupDefine};
        Project {
            version: "1.0.0".to_string(),
            base_package: "com.example".to_string(),
            biz_types: vec![],
            enums: vec![],
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
                        prop: "id".to_string(),
                        code: "ID".to_string(),
                        name: "主键".to_string(),
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
                        prop: "userName".to_string(),
                        code: "USER_NAME".to_string(),
                        name: "用户名".to_string(),
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
                    Field {
                        prop: "amount".to_string(),
                        code: "AMOUNT".to_string(),
                        name: "金额".to_string(),
                        data_type: DataType::Decimal,
                        length: None,
                        precision: Some(12),
                        scale: Some(2),
                        biz_type: None,
                        biz_type_data: None,
                        is_key: None,
                        not_null: None,
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
    fn test_sqlite_type_mapping() {
        assert_eq!(sqlite_type(DataType::Varchar), "TEXT");
        assert_eq!(sqlite_type(DataType::Clob), "TEXT");
        assert_eq!(sqlite_type(DataType::Tinyint), "INTEGER");
        assert_eq!(sqlite_type(DataType::Int), "INTEGER");
        assert_eq!(sqlite_type(DataType::Long), "INTEGER");
        assert_eq!(sqlite_type(DataType::Decimal), "TEXT"); // 精度保护
        assert_eq!(sqlite_type(DataType::Date), "TEXT");
        assert_eq!(sqlite_type(DataType::Datetime), "TEXT");
        assert_eq!(sqlite_type(DataType::Blob), "BLOB");
    }

    #[test]
    fn test_dataset_create_and_read() {
        let project = make_project();
        let dataset = Dataset::new(&project).expect("创建数据集失败");

        // 读取 schema 往返
        let read = dataset.get_project().expect("读取 schema 失败");
        assert_eq!(read.base_package, "com.example");
        assert_eq!(read.tables.len(), 1);
        assert_eq!(read.tables[0].code, "SYS_USER");
        assert_eq!(read.tables[0].fields.len(), 3);
    }

    #[test]
    fn test_dataset_create_table_executed() {
        let project = make_project();
        let dataset = Dataset::new(&project).expect("创建数据集失败");

        // 验证 SYS_USER 表已建(可插入数据)
        dataset
            .execute("INSERT INTO SYS_USER (ID, USER_NAME, AMOUNT) VALUES (1, 'admin', '99.50')")
            .expect("插入失败");

        // 验证查询
        let count: i64 = dataset
            .connection()
            .query_row("SELECT COUNT(*) FROM SYS_USER", [], |row| row.get(0))
            .expect("查询失败");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dataset_save_load() {
        let project = make_project();
        let dataset = Dataset::new(&project).expect("创建数据集失败");

        // 插入数据
        dataset
            .execute("INSERT INTO SYS_USER (ID, USER_NAME, AMOUNT) VALUES (1, 'admin', '99.50')")
            .expect("插入失败");

        // 保存到临时文件
        let tmp = "/tmp/aqua_test_dataset.db";
        let _ = std::fs::remove_file(tmp);
        dataset.save(tmp).expect("保存失败");

        // 重新加载
        let loaded = Dataset::load(tmp).expect("加载失败");
        let read = loaded.get_project().expect("读取失败");
        assert_eq!(read.tables[0].code, "SYS_USER");

        let count: i64 = loaded
            .connection()
            .query_row("SELECT COUNT(*) FROM SYS_USER", [], |row| row.get(0))
            .expect("查询失败");
        assert_eq!(count, 1, "加载后数据应保留");

        let _ = std::fs::remove_file(tmp);
    }
}
