//! Driver 模块 - 数据库连接层统一抽象。
//!
//! 支持两类驱动:
//! - 内置 native 驱动: MySQL(mysql_async), PostgreSQL(tokio-postgres)
//! - JDBC 驱动: Oracle/信创数据库(spawn connector.jar)

pub mod error;
mod factory;
mod jdbc;
mod mysql;
mod postgres;
mod trait_def;
pub mod types;

pub use error::DriverError;
pub use factory::create_driver;
pub use trait_def::Driver;
pub use types::{ColumnMeta, DbConfig, IndexMeta};
