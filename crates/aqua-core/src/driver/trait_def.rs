//! Driver trait 定义。

use super::error::Result;
use super::{ColumnMeta, IndexMeta};
use async_trait::async_trait;

/// 数据库驱动统一接口。
///
/// 支持两类实现:
/// - 内置 native 驱动(MySQL/PostgreSQL): 直接调用 Rust 异步驱动
/// - JDBC 驱动(Oracle/信创等): spawn connector.jar 子进程
///
/// # 设计原则
///
/// - 反解返回 aqua 逻辑类型(DataType),不返回物理类型
/// - 一次性连接,无连接池(低频工具可接受)
/// - 错误统一为 DriverError
#[async_trait]
pub trait Driver: Send + Sync {
    /// 测试数据库连接。
    async fn test_connection(&self) -> Result<()>;

    /// 列出所有表名。
    ///
    /// # 参数
    /// - `schema`: schema/database 名称(MySQL 用 database,Oracle/PG 用 schema)
    async fn list_tables(&self, schema: &str) -> Result<Vec<String>>;

    /// 获取表的列元数据(反解为 aqua 逻辑类型)。
    ///
    /// # 参数
    /// - `table`: 表名
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>>;

    /// 获取表的索引元数据。
    ///
    /// # 参数
    /// - `table`: 表名
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>>;

    // /// 查询数据行(用于数据集导入)。
    // ///
    // /// # 参数
    // /// - `sql`: 查询 SQL
    // /// - `limit`: 最大行数
    // async fn query_rows(&self, sql: &str, limit: usize) -> Result<Vec<Row>>;
}
