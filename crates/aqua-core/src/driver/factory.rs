//! Driver 工厂函数。

use std::path::PathBuf;

use super::jdbc::JdbcDriver;
use super::mysql::MysqlDriver;
use super::postgres::PostgresDriver;
use super::{DbConfig, Driver, DriverError};

/// 创建数据库驱动实例(工厂模式)。
///
/// # 参数
/// - `config`: 数据库连接配置
/// - `drivers_dir`: drivers/ 目录(JDBC 方言加载外置 jar 用;native 方言忽略)。`None` 则不加载外置驱动。
///
/// # 支持的方言
/// - "mysql": MySQL native 驱动
/// - "postgresql" | "postgres" | "pg": PostgreSQL native 驱动
/// - 其他: JDBC 驱动,spawn connector.jar
pub fn create_driver(
    config: DbConfig,
    drivers_dir: Option<PathBuf>,
) -> Result<Box<dyn Driver>, DriverError> {
    match config.dialect.as_str() {
        "mysql" => Ok(Box::new(MysqlDriver::new(&config)?)),
        "postgresql" | "postgres" | "pg" => Ok(Box::new(PostgresDriver::new(&config)?)),
        // 其他方言(Oracle/DM/KingBase/GBase/H2 等)走 JDBC connector.jar
        _ => Ok(Box::new(JdbcDriver::new(
            &config,
            "connector.jar",
            drivers_dir,
        ))),
    }
}
