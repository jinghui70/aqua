//! MySQL native 驱动实现。

use async_trait::async_trait;
use mysql_async::prelude::*;
use mysql_async::{Pool, OptsBuilder, Row};
use crate::driver::{Driver, DbConfig, ColumnMeta, IndexMeta, DriverError};
use crate::schema::DataType;
use std::collections::HashMap;

/// MySQL native 驱动。
pub struct MysqlDriver {
    pool: Pool,
    database: String,
}

impl MysqlDriver {
    /// 创建 MySQL 驱动实例。
    pub fn new(config: &DbConfig) -> Result<Self, DriverError> {
        let opts = OptsBuilder::default()
            .ip_or_hostname(&config.host)
            .tcp_port(config.port)
            .user(Some(&config.user))
            .pass(Some(&config.password))
            .db_name(Some(&config.database));

        let pool = Pool::new(opts.into());

        Ok(Self {
            pool,
            database: config.database.clone(),
        })
    }
}

#[async_trait]
impl Driver for MysqlDriver {
    async fn test_connection(&self) -> Result<(), DriverError> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        // 简单查询验证连接
        let _: Option<i32> = conn.query_first("SELECT 1").await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_tables(&self, schema: &str) -> Result<Vec<String>, DriverError> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let sql = "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = ? ORDER BY TABLE_NAME";
        let tables: Vec<String> = conn.exec(sql, (schema,)).await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        Ok(tables)
    }

    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, DriverError> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let sql = r"
            SELECT
                COLUMN_NAME,
                DATA_TYPE,
                CHARACTER_MAXIMUM_LENGTH,
                NUMERIC_PRECISION,
                NUMERIC_SCALE,
                IS_NULLABLE,
                COLUMN_KEY,
                COLUMN_DEFAULT,
                COLUMN_COMMENT
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
        ";

        let rows: Vec<Row> = conn.exec(sql, (&self.database, table)).await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        let mut columns = Vec::new();
        for row in rows {
            let name: String = row.get(0).unwrap();
            let data_type_str: String = row.get(1).unwrap();
            let char_length: Option<u32> = row.get(2).unwrap();
            let numeric_precision: Option<u32> = row.get(3).unwrap();
            let numeric_scale: Option<u32> = row.get(4).unwrap();
            let is_nullable: String = row.get(5).unwrap();
            let column_key: String = row.get(6).unwrap();
            let _default_value: Option<String> = row.get(7).unwrap();
            let comment: Option<String> = row.get(8).unwrap();

            let data_type = map_mysql_type(&data_type_str, char_length);
            let nullable = is_nullable == "YES";
            let is_key = column_key == "PRI";

            columns.push(ColumnMeta {
                name,
                data_type,
                length: char_length,
                precision: numeric_precision,
                scale: numeric_scale,
                nullable,
                is_key,
                default_value: None, // 简化处理
                comment,
            });
        }

        Ok(columns)
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>, DriverError> {
        let mut conn = self.pool.get_conn().await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let sql = r"
            SELECT
                INDEX_NAME,
                COLUMN_NAME,
                NON_UNIQUE
            FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            ORDER BY INDEX_NAME, SEQ_IN_INDEX
        ";

        let rows: Vec<Row> = conn.exec(sql, (&self.database, table)).await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        // 按 INDEX_NAME 分组
        let mut index_map: HashMap<String, (Vec<String>, bool)> = HashMap::new();
        for row in rows {
            let index_name: String = row.get(0).unwrap();
            let column_name: String = row.get(1).unwrap();
            let non_unique: i32 = row.get(2).unwrap();

            // 跳过主键索引(已在 ColumnMeta.is_key 处理)
            if index_name == "PRIMARY" {
                continue;
            }

            let unique = non_unique == 0;
            index_map.entry(index_name.clone())
                .or_insert_with(|| (Vec::new(), unique))
                .0.push(column_name);
        }

        let indexes = index_map.into_iter()
            .map(|(name, (fields, unique))| IndexMeta { name, fields, unique })
            .collect();

        Ok(indexes)
    }
}

/// MySQL 物理类型 → aqua 逻辑类型反解。
fn map_mysql_type(column_type: &str, length: Option<u32>) -> DataType {
    match column_type.to_uppercase().as_str() {
        "VARCHAR" | "CHAR" => DataType::Varchar,
        "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" => DataType::Clob,
        "TINYINT" => DataType::Tinyint,
        "INT" | "INTEGER" | "MEDIUMINT" | "SMALLINT" => DataType::Int,
        "BIGINT" => DataType::Long,
        "DECIMAL" | "NUMERIC" => DataType::Decimal,
        "DATE" => DataType::Date,
        "DATETIME" | "TIMESTAMP" => DataType::Datetime,
        "BLOB" | "BINARY" | "VARBINARY" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" => DataType::Blob,
        _ => {
            // 未知类型默认 VARCHAR
            if length.map_or(false, |l| l > 255) {
                DataType::Clob
            } else {
                DataType::Varchar
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_mysql_type() {
        assert_eq!(map_mysql_type("VARCHAR", Some(64)), DataType::Varchar);
        assert_eq!(map_mysql_type("TEXT", None), DataType::Clob);
        assert_eq!(map_mysql_type("TINYINT", None), DataType::Tinyint);
        assert_eq!(map_mysql_type("INT", None), DataType::Int);
        assert_eq!(map_mysql_type("BIGINT", None), DataType::Long);
        assert_eq!(map_mysql_type("DECIMAL", None), DataType::Decimal);
        assert_eq!(map_mysql_type("DATE", None), DataType::Date);
        assert_eq!(map_mysql_type("DATETIME", None), DataType::Datetime);
        assert_eq!(map_mysql_type("BLOB", None), DataType::Blob);
    }
}
