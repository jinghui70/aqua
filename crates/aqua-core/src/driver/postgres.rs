//! PostgreSQL native 驱动实现。

use crate::driver::{ColumnMeta, DbConfig, Driver, DriverError, IndexMeta};
use crate::schema::DataType;
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool};

/// PostgreSQL native 驱动。
pub struct PostgresDriver {
    pool: Pool,
    schema: String,
}

impl PostgresDriver {
    /// 创建 PostgreSQL 驱动实例。
    pub fn new(config: &DbConfig) -> Result<Self, DriverError> {
        let mut cfg = Config::new();
        cfg.host = Some(config.host.clone());
        cfg.port = Some(config.port);
        cfg.user = Some(config.user.clone());
        cfg.password = Some(config.password.clone());
        cfg.dbname = Some(config.database.clone());

        let pool = cfg
            .create_pool(None, tokio_postgres::NoTls)
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let schema = config
            .schema
            .clone()
            .unwrap_or_else(|| "public".to_string());

        Ok(Self { pool, schema })
    }
}

#[async_trait]
impl Driver for PostgresDriver {
    async fn test_connection(&self) -> Result<(), DriverError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_tables(&self, schema: &str) -> Result<Vec<String>, DriverError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(
                "SELECT table_name FROM information_schema.tables
                 WHERE table_schema = $1 AND table_type = 'BASE TABLE'
                 ORDER BY table_name",
                &[&schema],
            )
            .await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| r.get::<_, String>(0))
            .collect())
    }

    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, DriverError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(
                "SELECT column_name, data_type, character_maximum_length,
                        numeric_precision, numeric_scale, is_nullable, column_default
                 FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position",
                &[&self.schema, &table],
            )
            .await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        let mut columns = Vec::new();
        for row in &rows {
            let name: String = row.get(0);
            let data_type_str: String = row.get(1);
            let char_length: Option<i32> = row.get(2);
            let numeric_precision: Option<i32> = row.get(3);
            let numeric_scale: Option<i32> = row.get(4);
            let is_nullable: String = row.get(5);
            let default_value: Option<String> = row.get(6);

            let data_type = map_pg_type(&data_type_str);
            columns.push(ColumnMeta {
                name,
                data_type,
                length: char_length.map(|v| v as u32),
                precision: numeric_precision.map(|v| v as u32),
                scale: numeric_scale.map(|v| v as u32),
                nullable: is_nullable == "YES",
                is_key: false, // PG 主键需单独查询 pg_index,简化暂不处理
                default_value,
                comment: None, // PG 注释在 pg_description,简化暂不处理
            });
        }

        Ok(columns)
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>, DriverError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(
                "SELECT indexname, indexdef FROM pg_indexes
                 WHERE schemaname = $1 AND tablename = $2
                 ORDER BY indexname",
                &[&self.schema, &table],
            )
            .await
            .map_err(|e| DriverError::QueryFailed(e.to_string()))?;

        let mut indexes = Vec::new();
        for row in &rows {
            let name: String = row.get(0);
            let def: String = row.get(1);

            // 解析 indexdef: CREATE UNIQUE INDEX name ON schema.table USING btree (col1, col2)
            let unique = def.contains("UNIQUE");
            let fields = parse_index_fields(&def);
            indexes.push(IndexMeta {
                name,
                fields,
                unique,
            });
        }

        Ok(indexes)
    }
}

/// 解析 PG indexdef 提取字段列表。
fn parse_index_fields(def: &str) -> Vec<String> {
    // indexdef 格式: CREATE [UNIQUE] INDEX name ON schema.table USING btree (col1, col2)
    if let Some(start) = def.rfind('(') {
        if let Some(end) = def[start..].find(')') {
            let inner = &def[start + 1..start + end];
            return inner
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
    vec![]
}

/// PG 物理类型 -> aqua 逻辑类型反解。
fn map_pg_type(pg_type: &str) -> DataType {
    match pg_type.to_lowercase().as_str() {
        "character varying" | "char" | "bpchar" => DataType::Varchar,
        "text" => DataType::Clob,
        "smallint" | "int2" => DataType::Tinyint,
        "integer" | "int4" | "serial" => DataType::Int,
        "bigint" | "int8" | "bigserial" => DataType::Long,
        "numeric" | "decimal" => DataType::Decimal,
        "date" => DataType::Date,
        "timestamp" | "timestamp without time zone" | "timestamptz"
        | "timestamp with time zone" => DataType::Datetime,
        "bytea" => DataType::Blob,
        _ => DataType::Varchar, // 默认
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_pg_type() {
        assert_eq!(map_pg_type("character varying"), DataType::Varchar);
        assert_eq!(map_pg_type("text"), DataType::Clob);
        assert_eq!(map_pg_type("smallint"), DataType::Tinyint);
        assert_eq!(map_pg_type("integer"), DataType::Int);
        assert_eq!(map_pg_type("bigint"), DataType::Long);
        assert_eq!(map_pg_type("numeric"), DataType::Decimal);
        assert_eq!(map_pg_type("date"), DataType::Date);
        assert_eq!(map_pg_type("timestamp"), DataType::Datetime);
        assert_eq!(map_pg_type("bytea"), DataType::Blob);
    }

    #[test]
    fn test_parse_index_fields() {
        let def = "CREATE UNIQUE INDEX idx_user ON public.sys_user USING btree (user_name, status)";
        let fields = parse_index_fields(def);
        assert_eq!(fields, vec!["user_name", "status"]);
    }
}
