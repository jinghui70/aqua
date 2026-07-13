//! dataset 模块 - SQLite 数据集容器。
//!
//! 规则见 `docs/design.md` §4.5。
//! .aqua 文件 = SQLite,内含 _aqua_meta(schema.json) + 数据表。

use crate::schema::{DataType, Field, Project, Table};
use base64::prelude::*;
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

/// dataset 错误类型。
#[derive(Error, Debug)]
pub enum DatasetError {
    #[error("SQLite 错误: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("表不存在: {0}")]
    TableNotFound(String),
    #[error("数据集与项目表结构不一致: {0}")]
    SchemaMismatch(String),
    #[error("BLOB base64 解码失败: {0}")]
    Base64(String),
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

    /// 读取某表全部行为 JSON 对象数组(按字段类型转换)。
    pub fn read_table_rows(&self, table: &Table) -> Result<Vec<Map<String, Value>>, DatasetError> {
        let cols: Vec<String> = table.fields.iter().map(|f| f.code.to_uppercase()).collect();
        if cols.is_empty() {
            return Ok(vec![]);
        }
        let sql = format!(
            "SELECT {} FROM {}",
            cols.join(", "),
            table.code.to_uppercase()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let mut map = Map::new();
            for (i, f) in table.fields.iter().enumerate() {
                map.insert(f.code.to_uppercase(), row_to_json(f, row, i)?);
            }
            Ok(map)
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(Into::into)
    }

    /// 向某表插入多行(参数化,按字段类型绑定)。
    pub fn insert_rows(
        &self,
        table: &Table,
        rows: &[Map<String, Value>],
    ) -> Result<(), DatasetError> {
        if rows.is_empty() || table.fields.is_empty() {
            return Ok(());
        }
        let cols: Vec<String> = table.fields.iter().map(|f| f.code.to_uppercase()).collect();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table.code.to_uppercase(),
            cols.join(", "),
            placeholders.join(", ")
        );
        let mut stmt = self.conn.prepare(&sql)?;
        for row in rows {
            let params: Vec<Box<dyn rusqlite::ToSql>> = table
                .fields
                .iter()
                .map(|f| json_to_sql(f, row.get(&f.code.to_uppercase())))
                .collect::<Result<_, _>>()?;
            let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
            stmt.execute(refs.as_slice())?;
        }
        Ok(())
    }
}

/// rusqlite 行值 → JSON(§4.5:整数用 number,其余 TEXT 用 string,BLOB base64,空 null)。
fn row_to_json(field: &Field, row: &rusqlite::Row, idx: usize) -> SqlResult<Value> {
    Ok(match field.data_type {
        DataType::Tinyint | DataType::Int | DataType::Long => {
            let v: Option<i64> = row.get(idx)?;
            v.map(|n| Value::Number(n.into())).unwrap_or(Value::Null)
        }
        DataType::Blob => {
            let v: Option<Vec<u8>> = row.get(idx)?;
            v.map(|b| Value::String(BASE64_STANDARD.encode(b)))
                .unwrap_or(Value::Null)
        }
        // VARCHAR/CLOB/DECIMAL/DATE/DATETIME 均存 TEXT
        _ => {
            let v: Option<String> = row.get(idx)?;
            v.map(Value::String).unwrap_or(Value::Null)
        }
    })
}

/// JSON 值 → rusqlite 参数(按字段类型绑定)。
fn json_to_sql(
    field: &Field,
    v: Option<&Value>,
) -> Result<Box<dyn rusqlite::ToSql>, DatasetError> {
    let v = v.unwrap_or(&Value::Null);
    Ok(match (field.data_type, v) {
        (_, Value::Null) => Box::new(rusqlite::types::Null),
        (DataType::Tinyint | DataType::Int | DataType::Long, Value::Number(n)) => {
            Box::new(n.as_i64().unwrap_or(0))
        }
        (DataType::Blob, Value::String(s)) => Box::new(
            BASE64_STANDARD
                .decode(s)
                .map_err(|e| DatasetError::Base64(e.to_string()))?,
        ),
        (_, Value::String(s)) => Box::new(s.clone()),
        // 容错:类型与值不完全匹配时按字符串存
        (_, other) => Box::new(other.to_string()),
    })
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

/// 加载数据集文件(按扩展名分派 JSON / SQLite),校验后返回条目。
pub fn load_dataset(path: &str, project: &Project) -> Result<Vec<DatasetEntry>, DatasetError> {
    if path.ends_with(".json") {
        let content = std::fs::read_to_string(path)?;
        let entries: Vec<DatasetEntry> = serde_json::from_str(&content)?;
        validate_against(project, &entries)?;
        Ok(entries)
    } else {
        let ds = Dataset::load(path)?;
        let mut entries = Vec::new();
        for table in &project.tables {
            let rows = ds.read_table_rows(table)?;
            entries.push(DatasetEntry {
                table: table.code.clone(),
                data: rows,
            });
        }
        Ok(entries)
    }
}

/// 保存数据集文件(按扩展名分派)。SQLite 全量重建,不追加。
pub fn save_dataset(
    path: &str,
    project: &Project,
    entries: &[DatasetEntry],
) -> Result<(), DatasetError> {
    validate_against(project, entries)?;
    if path.ends_with(".json") {
        std::fs::write(path, serde_json::to_string_pretty(entries)?)?;
        Ok(())
    } else {
        let ds = Dataset::new(project)?; // 内存建表
        for entry in entries {
            let table = project
                .tables
                .iter()
                .find(|t| t.code == entry.table)
                .ok_or_else(|| DatasetError::TableNotFound(entry.table.clone()))?;
            ds.insert_rows(table, &entry.data)?;
        }
        let _ = std::fs::remove_file(path); // 避免 backup 叠加旧数据
        ds.save(path)?;
        Ok(())
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

/// 数据集 JSON 条目(§4.5 JSON 格式)。行为对象(key=字段 code),
/// 非对象行在反序列化阶段即报错,避免静默丢行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub table: String,
    pub data: Vec<Map<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_project() -> Project {
        use crate::schema::{Field, GroupDefine};
        Project {
            version: "1.0.0".to_string(),
            name: None,
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

    fn sample_entries() -> Vec<DatasetEntry> {
        let mut row = Map::new();
        row.insert("ID".into(), Value::Number(1.into()));
        row.insert("USER_NAME".into(), Value::String("admin".into()));
        row.insert("AMOUNT".into(), Value::String("99.50".into()));
        let mut row2 = Map::new();
        row2.insert("ID".into(), Value::Number(2.into()));
        row2.insert("USER_NAME".into(), Value::Null); // null 保留
        row2.insert("AMOUNT".into(), Value::String("1234567890.12".into()));
        vec![DatasetEntry {
            table: "SYS_USER".into(),
            data: vec![row, row2],
        }]
    }

    #[test]
    fn test_json_dataset_roundtrip() {
        let project = make_project();
        let entries = sample_entries();
        let tmp = "/tmp/aqua_test_dataset_rt.json";
        let _ = std::fs::remove_file(tmp);

        save_dataset(tmp, &project, &entries).expect("保存 JSON 失败");
        let loaded = load_dataset(tmp, &project).expect("加载 JSON 失败");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].data.len(), 2);
        // DECIMAL 保精度(字符串)
        assert_eq!(loaded[0].data[1]["AMOUNT"], Value::String("1234567890.12".into()));
        // null 保留
        assert_eq!(loaded[0].data[1]["USER_NAME"], Value::Null);
        // INT 为数字
        assert_eq!(loaded[0].data[0]["ID"], Value::Number(1.into()));
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn test_sqlite_dataset_roundtrip() {
        let project = make_project();
        let entries = sample_entries();
        let tmp = "/tmp/aqua_test_dataset_rt.db";
        let _ = std::fs::remove_file(tmp);

        save_dataset(tmp, &project, &entries).expect("保存 SQLite 失败");
        let loaded = load_dataset(tmp, &project).expect("加载 SQLite 失败");

        let user = loaded.iter().find(|e| e.table == "SYS_USER").expect("缺表");
        assert_eq!(user.data.len(), 2);
        // DECIMAL 精度不丢(TEXT 存储)
        assert_eq!(user.data[1]["AMOUNT"], Value::String("1234567890.12".into()));
        // null 往返
        assert_eq!(user.data[1]["USER_NAME"], Value::Null);
        // INT 仍为数字
        assert_eq!(user.data[0]["ID"], Value::Number(1.into()));
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn test_validate_schema_mismatch() {
        let project = make_project();
        // 未知表
        let bad_table = vec![DatasetEntry {
            table: "NOT_EXIST".into(),
            data: vec![],
        }];
        assert!(matches!(
            validate_against(&project, &bad_table),
            Err(DatasetError::TableNotFound(_))
        ));
        // 未知字段
        let mut row = Map::new();
        row.insert("BAD_FIELD".into(), Value::Null);
        let bad_field = vec![DatasetEntry {
            table: "SYS_USER".into(),
            data: vec![row],
        }];
        assert!(matches!(
            validate_against(&project, &bad_field),
            Err(DatasetError::SchemaMismatch(_))
        ));
    }
}
