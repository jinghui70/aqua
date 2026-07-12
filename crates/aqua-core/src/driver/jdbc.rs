//! JDBC 驱动实现 - 通过 spawn connector.jar 访问 Oracle/信创数据库。
//!
//! 通信协议(architecture.md §2): 一次性命令,stdin JSON 请求 -> stdout JSON 响应 -> exit。
//! connector.jar (Java) 负责实际 JDBC 连接 + 反解,本驱动只做子进程通信。

use crate::driver::{ColumnMeta, DbConfig, Driver, DriverError, IndexMeta};
use crate::schema::DataType;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// JDBC 驱动(spawn connector.jar)。
pub struct JdbcDriver {
    config: DbConfig,
    connector_path: String,
}

impl JdbcDriver {
    /// 创建 JDBC 驱动。
    ///
    /// `connector_path`: connector.jar 路径(默认 "connector.jar")。
    pub fn new(config: &DbConfig, connector_path: &str) -> Self {
        Self {
            config: config.clone(),
            connector_path: connector_path.to_string(),
        }
    }

    /// 调用 connector.jar,发送请求,返回响应 JSON。
    async fn call(&self, action: &str, extra: Option<Value>) -> Result<Value, DriverError> {
        let mut request = json!({
            "action": action,
            "dialect": self.config.dialect,
            "host": self.config.host,
            "port": self.config.port,
            "user": self.config.user,
            "password": self.config.password,
            "database": self.config.database,
        });

        if let Some(extra_val) = extra {
            if let (Some(req_map), Some(extra_map)) =
                (request.as_object_mut(), extra_val.as_object())
            {
                req_map.extend(extra_map.clone());
            }
        }

        let mut child = Command::new("java")
            .arg("-jar")
            .arg(&self.connector_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                DriverError::ConnectionFailed(format!("启动 connector 失败(需 JDK 17+): {}", e))
            })?;

        // 写 stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(request.to_string().as_bytes())
                .await
                .map_err(|e| DriverError::ConnectionFailed(format!("写 stdin 失败: {}", e)))?;
        }

        // 读 stdout
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| DriverError::ConnectionFailed(format!("connector 执行失败: {}", e)))?;

        if !output.status.success() {
            return Err(DriverError::ConnectionFailed(format!(
                "connector 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let response: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| DriverError::ConnectionFailed(format!("connector 响应解析失败: {}", e)))?;

        // 检查错误响应
        if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
            return Err(DriverError::QueryFailed(error.to_string()));
        }

        Ok(response)
    }
}

#[async_trait]
impl Driver for JdbcDriver {
    async fn test_connection(&self) -> Result<(), DriverError> {
        self.call("testConnection", None).await?;
        Ok(())
    }

    async fn list_tables(&self, schema: &str) -> Result<Vec<String>, DriverError> {
        let resp = self
            .call("listTables", Some(json!({ "schema": schema })))
            .await?;

        let tables = resp
            .get("tables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(tables)
    }

    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, DriverError> {
        let resp = self
            .call("getColumns", Some(json!({ "table": table })))
            .await?;

        let columns = resp
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_column_meta).collect())
            .unwrap_or_default();

        Ok(columns)
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>, DriverError> {
        let resp = self
            .call("listIndexes", Some(json!({ "table": table })))
            .await?;

        let indexes = resp
            .get("indexes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_index_meta).collect())
            .unwrap_or_default();

        Ok(indexes)
    }
}

/// 解析 connector 返回的列元数据。
fn parse_column_meta(v: &Value) -> Option<ColumnMeta> {
    Some(ColumnMeta {
        name: v.get("name")?.as_str()?.to_string(),
        data_type: parse_data_type(v.get("dataType")?.as_str()?),
        length: v.get("length").and_then(|x| x.as_u64()).map(|n| n as u32),
        precision: v
            .get("precision")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32),
        scale: v.get("scale").and_then(|x| x.as_u64()).map(|n| n as u32),
        nullable: v.get("nullable").and_then(|x| x.as_bool()).unwrap_or(true),
        is_key: v.get("isKey").and_then(|x| x.as_bool()).unwrap_or(false),
        default_value: v
            .get("defaultValue")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        comment: v
            .get("comment")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
    })
}

/// 解析 connector 返回的索引元数据。
fn parse_index_meta(v: &Value) -> Option<IndexMeta> {
    Some(IndexMeta {
        name: v.get("name")?.as_str()?.to_string(),
        fields: v
            .get("fields")?
            .as_array()?
            .iter()
            .filter_map(|f| f.as_str().map(|s| s.to_string()))
            .collect(),
        unique: v.get("unique").and_then(|x| x.as_bool()).unwrap_or(false),
    })
}

/// connector 返回的逻辑类型字符串 -> DataType。
fn parse_data_type(s: &str) -> DataType {
    match s.to_uppercase().as_str() {
        "VARCHAR" => DataType::Varchar,
        "CLOB" => DataType::Clob,
        "TINYINT" => DataType::Tinyint,
        "INT" => DataType::Int,
        "LONG" => DataType::Long,
        "DECIMAL" => DataType::Decimal,
        "DATE" => DataType::Date,
        "DATETIME" => DataType::Datetime,
        "BLOB" => DataType::Blob,
        _ => DataType::Varchar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_type() {
        assert_eq!(parse_data_type("VARCHAR"), DataType::Varchar);
        assert_eq!(parse_data_type("CLOB"), DataType::Clob);
        assert_eq!(parse_data_type("INT"), DataType::Int);
        assert_eq!(parse_data_type("LONG"), DataType::Long);
        assert_eq!(parse_data_type("DECIMAL"), DataType::Decimal);
        assert_eq!(parse_data_type("DATETIME"), DataType::Datetime);
        assert_eq!(parse_data_type("BLOB"), DataType::Blob);
    }

    #[test]
    fn test_parse_column_meta() {
        let v = json!({
            "name": "USER_NAME",
            "dataType": "VARCHAR",
            "length": 64,
            "nullable": false,
            "isKey": false,
            "comment": "用户名"
        });

        let col = parse_column_meta(&v).unwrap();
        assert_eq!(col.name, "USER_NAME");
        assert_eq!(col.data_type, DataType::Varchar);
        assert_eq!(col.length, Some(64));
        assert_eq!(col.nullable, false);
        assert_eq!(col.comment, Some("用户名".to_string()));
    }

    #[test]
    fn test_parse_index_meta() {
        let v = json!({
            "name": "IDX_USER_NAME",
            "fields": ["USER_NAME", "STATUS"],
            "unique": true
        });

        let idx = parse_index_meta(&v).unwrap();
        assert_eq!(idx.name, "IDX_USER_NAME");
        assert_eq!(idx.fields, vec!["USER_NAME", "STATUS"]);
        assert_eq!(idx.unique, true);
    }
}
