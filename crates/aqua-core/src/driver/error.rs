//! Driver 错误类型。

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("查询失败: {0}")]
    QueryFailed(String),

    #[error("不支持的数据库方言: {0}")]
    UnsupportedDialect(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("类型映射失败: {0}")]
    TypeMappingError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DriverError>;
