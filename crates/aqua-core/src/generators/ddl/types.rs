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
            // JDBC 数据库
            "oracle" | "dm" | "kingbase" | "gbase" | "h2" | "sqlserver" | "oceanbase" | "tidb"
            | "gaussdb" => Some(Self::Jdbc {
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
    /// 是否包含 DROP TABLE IF EXISTS(默认 true)
    pub drop_if_exist: bool,
}

impl Default for DdlOptions {
    fn default() -> Self {
        Self {
            dialect: Dialect::Mysql,
            tables: None,
            group: None,
            drop_if_exist: true,
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

        // === JDBC 数据库: 核心层硬编码 + 兼容层复用 ===
        Dialect::Jdbc { name } => match name.as_str() {
            "oracle" => map_oracle(data_type, length, precision, scale),
            "h2" => map_h2(data_type, length, precision, scale),
            "dm" => map_dm(data_type, length, precision, scale),
            // KingBase 兼容 PostgreSQL,复用其映射
            "kingbase" => map_postgresql(data_type, length, precision, scale),
            "gbase" => map_gbase(data_type, length, precision, scale),
            "sqlserver" => map_sqlserver(data_type, length, precision, scale),
            // 兼容层复用 MySQL/PostgreSQL
            "oceanbase" | "tidb" => map_mysql(data_type, length, precision, scale),
            "gaussdb" => map_postgresql(data_type, length, precision, scale),
            _ => unreachable!("未覆盖的 JDBC 数据库: {}", name),
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

/// DM 达梦类型映射(硬编码)。
fn map_dm(
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
        DataType::Datetime => "DATETIME".to_string(),
        DataType::Blob => "BLOB".to_string(),
    }
}

/// GBase 南大通用类型映射(硬编码)。
fn map_gbase(
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

/// SQL Server 类型映射(硬编码)。
fn map_sqlserver(
    data_type: DataType,
    length: Option<u32>,
    precision: Option<u32>,
    scale: Option<u32>,
) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR({})", length.unwrap_or(255)),
        DataType::Clob => "VARCHAR(MAX)".to_string(),
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
        DataType::Datetime => "DATETIME2".to_string(),
        DataType::Blob => "VARBINARY(MAX)".to_string(),
    }
}

/// SQL 字符串字面量转义: 单引号 -> ''。
pub fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::list_dialects;
    use crate::schema::DataType;

    #[test]
    fn test_map_type_all_databases_covered() {
        // 所有数据库的每种 DataType 都能映射,不 panic,不 UNKNOWN
        let types = [
            DataType::Varchar,
            DataType::Clob,
            DataType::Tinyint,
            DataType::Int,
            DataType::Long,
            DataType::Decimal,
            DataType::Date,
            DataType::Datetime,
            DataType::Blob,
        ];
        for db in list_dialects() {
            let dialect =
                Dialect::parse(db.name).unwrap_or_else(|| panic!("{} parse 失败", db.name));
            for dt in types {
                let s = map_type(dt, Some(64), Some(10), Some(2), &dialect);
                assert!(!s.starts_with("UNKNOWN"), "{} {:?} -> {}", db.name, dt, s);
            }
        }
    }

    #[test]
    fn test_map_mysql() {
        assert_eq!(
            map_type(DataType::Varchar, Some(64), None, None, &Dialect::Mysql),
            "VARCHAR(64)"
        );
        assert_eq!(
            map_type(DataType::Long, None, None, None, &Dialect::Mysql),
            "BIGINT"
        );
        assert_eq!(
            map_type(DataType::Decimal, None, Some(10), Some(2), &Dialect::Mysql),
            "DECIMAL(10, 2)"
        );
    }

    #[test]
    fn test_map_sqlserver() {
        let d = Dialect::Jdbc {
            name: "sqlserver".to_string(),
        };
        assert_eq!(
            map_type(DataType::Clob, None, None, None, &d),
            "VARCHAR(MAX)"
        );
        assert_eq!(
            map_type(DataType::Datetime, None, None, None, &d),
            "DATETIME2"
        );
        assert_eq!(
            map_type(DataType::Blob, None, None, None, &d),
            "VARBINARY(MAX)"
        );
    }

    #[test]
    fn test_map_compatible_reuse() {
        // oceanbase/tidb 复用 mysql,gaussdb 复用 postgresql
        let ob = Dialect::Jdbc {
            name: "oceanbase".to_string(),
        };
        let mysql = map_type(DataType::Long, None, None, None, &Dialect::Mysql);
        assert_eq!(map_type(DataType::Long, None, None, None, &ob), mysql);

        let gauss = Dialect::Jdbc {
            name: "gaussdb".to_string(),
        };
        let pg = map_type(DataType::Blob, None, None, None, &Dialect::Postgresql);
        assert_eq!(map_type(DataType::Blob, None, None, None, &gauss), pg);
    }
}
