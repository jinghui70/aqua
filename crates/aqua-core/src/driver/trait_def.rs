//! Driver trait 定义。

use super::error::Result;
use super::{ColumnMeta, IndexMeta, TableInfo};
use async_trait::async_trait;
use serde_json::Map;

/// 数据库驱动统一接口。
#[async_trait]
pub trait Driver: Send + Sync {
    async fn test_connection(&self) -> Result<()>;
    async fn list_tables(&self, schema: &str) -> Result<Vec<TableInfo>>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>>;
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>>;

    /// 查询表全部行(导入数据集用)。返回 Vec<Map<列名, 值>>。
    async fn query_table_rows(&self, table: &str) -> Result<Vec<Map<String, serde_json::Value>>>;

    /// 执行 UPDATE/INSERT/TRUNCATE(导出数据集用)。返回影响行数。
    async fn execute_update(&self, sql: &str) -> Result<usize>;
}
