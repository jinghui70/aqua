//! DDL 生成器类型定义与选项。

use crate::schema::DataType;

/// 数据库方言分类。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Dialect {
    /// 内置 native 驱动(类型映射硬编码)。
    #[default]
    Mysql,
    Postgresql,

    /// 外置 JDBC 驱动(类型映射从 connector.jar 或配置文件加载)。
    /// name: 方言标识(如 "oracle", "dm", "kingbase"),传给 connector.jar。
    /// mappings: 可选的类型映射覆盖(未来扩展,当前从 connector 获取)。
    Jdbc {
        name: String,
    },
}

impl Dialect {
    /// 从字符串创建方言(CLI 用)。
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mysql" => Some(Self::Mysql),
            "postgresql" | "postgres" | "pg" => Some(Self::Postgresql),
            // JDBC 方言
            "oracle" | "dm" | "kingbase" | "gbase" | "h2" => Some(Self::Jdbc {
                name: s.to_lowercase(),
            }),
            _ => None,
        }
    }

    /// 是否为内置方言。
    pub const fn is_native(&self) -> bool {
        matches!(self, Self::Mysql | Self::Postgresql)
    }
}

/// DDL 生成选项。
#[derive(Debug, Clone)]
pub struct DdlOptions {
    /// 目标方言,默认 MySQL。
    pub dialect: Dialect,
    /// 表名过滤(与 group 互斥),不指定则全部表。
    pub tables: Option<Vec<String>>,
    /// 分组过滤(与 tables 互斥)。
    pub group: Option<String>,
}

impl Default for DdlOptions {
    fn default() -> Self {
        Self {
            dialect: Dialect::Mysql,
            tables: None,
            group: None,
        }
    }
}

/// 逻辑类型映射为物理类型。
///
/// **内置方言**(MySQL/PostgreSQL): 硬编码完整映射。
/// **JDBC 方言**: 提供 Oracle/H2 作为参考实现,实际应从 connector.jar 获取映射。
pub fn map_type(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
    dialect: &Dialect,
) -> String {
    match dialect {
        // === 内置方言: 完整实现 ===
        Dialect::Mysql => map_mysql(data_type, length, precision, scale),
        Dialect::Postgresql => map_postgresql(data_type, length, precision, scale),

        // === JDBC 方言: 示例实现 ===
        Dialect::Jdbc { name } => match name.as_str() {
            "oracle" => map_oracle(data_type, length, precision, scale),
            "h2" => map_h2(data_type, length, precision, scale),
            // 其他 JDBC 方言(dm/kingbase/gbase 等)应从 connector.jar 动态获取
            _ => format!("UNKNOWN_{:?}", data_type), // 占位,实际应报错或查配置
        },
    }
}

/// MySQL 类型映射(内置完整实现)。
fn map_mysql(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR({})", length.unwrap_or(255)),
        DataType::Clob => "TEXT".to_string(),
        DataType::Tinyint => "TINYINT".to_string(),
        DataType::Int => "INT".to_string(),
        DataType::Long => "BIGINT".to_string(),
        DataType::Decimal => {
            if let Some(p) = precision {
                format!("DECIMAL({}, {})", p, scale.unwrap_or(0))
            } else {
                "DECIMAL".to_string()
            }
        }
        DataType::Date => "DATE".to_string(),
        DataType::Datetime => "DATETIME".to_string(),
        DataType::Blob => "BLOB".to_string(),
    }
}

/// PostgreSQL 类型映射(内置完整实现)。
fn map_postgresql(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR({})", length.unwrap_or(255)),
        DataType::Clob => "TEXT".to_string(),
        DataType::Tinyint => "SMALLINT".to_string(),
        DataType::Int => "INTEGER".to_string(),
        DataType::Long => "BIGINT".to_string(),
        DataType::Decimal => {
            if let Some(p) = precision {
                format!("NUMERIC({}, {})", p, scale.unwrap_or(0))
            } else {
                "NUMERIC".to_string()
            }
        }
        DataType::Date => "DATE".to_string(),
        DataType::Datetime => "TIMESTAMP".to_string(),
        DataType::Blob => "BYTEA".to_string(),
    }
}

/// Oracle 类型映射(JDBC 示例,实际应从 connector 获取)。
fn map_oracle(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR2({})", length.unwrap_or(255)),
        DataType::Clob => "CLOB".to_string(),
        DataType::Tinyint => "NUMBER(3)".to_string(),
        DataType::Int => "NUMBER(10)".to_string(),
        DataType::Long => "NUMBER(19)".to_string(),
        DataType::Decimal => {
            if let Some(p) = precision {
                format!("NUMBER({}, {})", p, scale.unwrap_or(0))
            } else {
                "NUMBER".to_string()
            }
        }
        DataType::Date => "DATE".to_string(),
        DataType::Datetime => "TIMESTAMP".to_string(),
        DataType::Blob => "BLOB".to_string(),
    }
}

/// H2 类型映射(JDBC 示例,实际应从 connector 获取)。
fn map_h2(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR({})", length.unwrap_or(255)),
        DataType::Clob => "CLOB".to_string(),
        DataType::Tinyint => "TINYINT".to_string(),
        DataType::Int => "INT".to_string(),
        DataType::Long => "BIGINT".to_string(),
        DataType::Decimal => {
            if let Some(p) = precision {
                format!("DECIMAL({}, {})", p, scale.unwrap_or(0))
            } else {
                "DECIMAL".to_string()
            }
        }
        DataType::Date => "DATE".to_string(),
        DataType::Datetime => "TIMESTAMP".to_string(),
        DataType::Blob => "BLOB".to_string(),
    }
}

/// SQL 字符串字面量转义: 单引号 -> ''。
pub fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}
