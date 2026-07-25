//! Driver trait 定义。

use super::error::Result;
use super::{ColumnMeta, IndexMeta, TableInfo, TableMeta};
use async_trait::async_trait;
use serde_json::Map;

/// 数据库驱动统一接口。
#[async_trait]
pub trait Driver: Send + Sync {
    async fn test_connection(&self) -> Result<()>;
    async fn list_tables(&self, schema: &str) -> Result<Vec<TableInfo>>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>>;
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>>;

    /// 批量反解多表(列+索引),供从数据库导入 schema 用。
    ///
    /// 默认逐表调用 `get_columns`/`list_indexes`——native(MySQL/PG)走连接池跨调用复用连接,
    /// 无额外开销,故用默认实现即可。JDBC 覆写为**单次 spawn**,消除导 N 表的 2N 次 JVM 冷启动。
    async fn import_tables(&self, tables: &[String]) -> Result<Vec<TableMeta>> {
        let mut out = Vec::with_capacity(tables.len());
        for t in tables {
            let columns = self.get_columns(t).await?;
            let indexes = self.list_indexes(t).await?;
            out.push(TableMeta {
                name: t.clone(),
                columns,
                indexes,
            });
        }
        Ok(out)
    }

    /// 查询表全部行(导入数据集用)。返回 Vec<Map<列名, 值>>。
    async fn query_table_rows(&self, table: &str) -> Result<Vec<Map<String, serde_json::Value>>>;

    /// 执行 UPDATE/INSERT/TRUNCATE(导出数据集用)。返回影响行数。
    async fn execute_update(&self, sql: &str) -> Result<usize>;
}
