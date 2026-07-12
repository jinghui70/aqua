//! Driver 工厂函数。

use super::{DbConfig, Driver, DriverError};

/// 创建数据库驱动实例(工厂模式)。
///
/// # 参数
/// - `config`: 数据库连接配置
///
/// # 返回
/// - `Box<dyn Driver>`: trait object,具体类型由 dialect 决定
///
/// # 支持的方言
/// - "mysql": MySQL native 驱动(后续任务实现)
/// - "postgresql" | "postgres" | "pg": PostgreSQL native 驱动(后续任务实现)
/// - 其他: JDBC 驱动,spawn connector.jar(后续任务实现)
///
/// # 示例
///
/// ```ignore
/// let config = DbConfig {
///     dialect: "mysql".to_string(),
///     host: "localhost".to_string(),
///     port: 3306,
///     user: "root".to_string(),
///     password: "password".to_string(),
///     database: "test".to_string(),
///     schema: None,
/// };
///
/// let driver = create_driver(config)?;
/// driver.test_connection().await?;
/// let tables = driver.list_tables("test").await?;
/// ```
pub fn create_driver(_config: DbConfig) -> Result<Box<dyn Driver>, DriverError> {
    // 当前占位实现,后续任务补充:
    // - 07-12-driver-mysql: MySQL native 驱动
    // - 07-12-driver-postgres: PostgreSQL native 驱动
    // - 07-12-driver-jdbc: JDBC 驱动(spawn connector.jar)

    Err(DriverError::UnsupportedDialect(
        "Driver 尚未实现,后续任务补充".to_string(),
    ))
}
