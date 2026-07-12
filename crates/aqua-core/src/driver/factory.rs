//! Driver 工厂函数。

use super::mysql::MysqlDriver;
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
/// - "mysql": MySQL native 驱动
/// - "postgresql" | "postgres" | "pg": PostgreSQL native 驱动(待实现)
/// - 其他: JDBC 驱动,spawn connector.jar(待实现)
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
pub fn create_driver(config: DbConfig) -> Result<Box<dyn Driver>, DriverError> {
    match config.dialect.as_str() {
        "mysql" => Ok(Box::new(MysqlDriver::new(&config)?)),
        "postgresql" | "postgres" | "pg" => Err(DriverError::UnsupportedDialect(
            "PostgreSQL 驱动待实现(07-12-driver-postgres)".to_string(),
        )),
        _ => Err(DriverError::UnsupportedDialect(format!(
            "JDBC 驱动待实现(07-12-driver-jdbc): {}",
            config.dialect
        ))),
    }
}
