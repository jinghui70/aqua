//! Driver 模块 - 数据库连接层统一抽象。
//!
//! 支持两类驱动:
//! - 内置 native 驱动: MySQL(mysql_async), PostgreSQL(tokio-postgres)
//! - JDBC 驱动: Oracle/信创数据库(spawn connector.jar)

pub mod types;
pub mod error;
mod trait_def;
mod factory;
mod mysql;

pub use types::{DbConfig, ColumnMeta, IndexMeta};
pub use error::DriverError;
pub use trait_def::Driver;
pub use factory::create_driver;
