//! 数据库清单 - aqua 支持的数据库元信息(供前端列表 + 生成/反解路由)。
//!
//! 静态硬编码,与 connector 侧 DialectRegistry 人工同步。
//! 用户可见层叫"数据库";代码层 `Dialect` 类型名保留(SQL 方言行业术语)。

/// 数据库分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbCategory {
    /// Rust native 驱动(MySQL/PostgreSQL)。
    Native,
    /// Java JDBC 驱动(connector.jar)。
    Jdbc,
}

/// 单个数据库的元信息。
#[derive(Debug, Clone)]
pub struct DialectInfo {
    /// 标识(如 "oracle"),与 `DbConfig.dialect`、connector `Dialect.name()` 对应。
    pub name: &'static str,
    /// 显示名(如 "Oracle")。
    pub label: &'static str,
    /// 分类。
    pub category: DbCategory,
    /// 默认端口。
    pub default_port: u16,
    /// 是否需要 schema(MySQL 用 database,Oracle/PG 用 schema)。
    pub needs_schema: bool,
    /// 生成时复用谁的映射(兼容层): "mysql"/"postgresql"。None=独立映射。
    pub generate_as: Option<&'static str>,
    /// JDBC Driver 全限定类名(None=native)。
    pub driver_class: Option<&'static str>,
    /// 是否有反解 Dialect 实现(connector 侧)。
    pub reverse_supported: bool,
    /// 驱动是否内置(native 或 shade 进 connector.jar)。
    pub builtin_driver: bool,
}

/// 全部支持的数据库清单(8 个)。
///
/// 内置: mysql/postgresql/h2
/// 外置 JDBC: oracle/dm/kingbase/gbase/sqlserver
pub static ALL_DATABASES: &[DialectInfo] = &[
    // === native 固定,不可卸载 ===
    DialectInfo {
        name: "mysql",
        label: "MySQL",
        category: DbCategory::Native,
        default_port: 3306,
        needs_schema: false,
        generate_as: None,
        driver_class: None,
        reverse_supported: true,
        builtin_driver: true,
    },
    DialectInfo {
        name: "postgresql",
        label: "PostgreSQL",
        category: DbCategory::Native,
        default_port: 5432,
        needs_schema: true,
        generate_as: None,
        driver_class: None,
        reverse_supported: true,
        builtin_driver: true,
    },
    // === JDBC 内置(H2 驱动 shade 进 connector.jar) ===
    DialectInfo {
        name: "h2",
        label: "H2",
        category: DbCategory::Jdbc,
        default_port: 8082,
        needs_schema: false,
        generate_as: None,
        driver_class: Some("org.h2.Driver"),
        reverse_supported: true,
        builtin_driver: true,
    },
    // === JDBC 核心(驱动用户自备) ===
    DialectInfo {
        name: "oracle",
        label: "Oracle",
        category: DbCategory::Jdbc,
        default_port: 1521,
        needs_schema: true,
        generate_as: None,
        driver_class: Some("oracle.jdbc.OracleDriver"),
        reverse_supported: true,
        builtin_driver: false,
    },
    DialectInfo {
        name: "dm",
        label: "DM 达梦",
        category: DbCategory::Jdbc,
        default_port: 5236,
        needs_schema: true,
        generate_as: None,
        driver_class: Some("dm.jdbc.driver.DmDriver"),
        reverse_supported: true, // 通用兜底反解,未实测
        builtin_driver: false,
    },
    DialectInfo {
        name: "kingbase",
        label: "KingBase",
        category: DbCategory::Jdbc,
        default_port: 54321,
        needs_schema: true,
        generate_as: None,
        driver_class: Some("com.kingbase8.Driver"),
        reverse_supported: true, // 通用兜底反解,未实测
        builtin_driver: false,
    },
    DialectInfo {
        name: "gbase",
        label: "GBase",
        category: DbCategory::Jdbc,
        default_port: 5258,
        needs_schema: true,
        generate_as: None,
        driver_class: Some("com.gbase.jdbc.Driver"),
        reverse_supported: true, // 通用兜底反解,未实测
        builtin_driver: false,
    },
    DialectInfo {
        name: "sqlserver",
        label: "SQL Server",
        category: DbCategory::Jdbc,
        default_port: 1433,
        needs_schema: false,
        generate_as: None,
        driver_class: Some("com.microsoft.sqlserver.jdbc.SQLServerDriver"),
        reverse_supported: true, // 通用兜底反解,未实测
        builtin_driver: false,
    },
];

/// 查询全部数据库元信息。
pub fn list_dialects() -> &'static [DialectInfo] {
    ALL_DATABASES
}

/// 按名查数据库元信息。
pub fn find_dialect(name: &str) -> Option<&'static DialectInfo> {
    ALL_DATABASES.iter().find(|d| d.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_dialects_count() {
        assert_eq!(ALL_DATABASES.len(), 8);
    }

    #[test]
    fn test_find_dialect() {
        assert_eq!(find_dialect("oracle").unwrap().default_port, 1521);
        assert!(find_dialect("nonexistent").is_none());
    }

    #[test]
    fn test_names_unique() {
        let mut names: Vec<_> = ALL_DATABASES.iter().map(|d| d.name).collect();
        names.sort_unstable();
        let len = names.len();
        names.dedup();
        assert_eq!(names.len(), len, "数据库名重复");
    }

    #[test]
    fn test_reverse_supported_set() {
        // 全部 8 个库都支持反解(native 已验证 + H2/Oracle 已验证 + 信创/SQLServer 通用兜底未实测)
        let rev: Vec<_> = ALL_DATABASES
            .iter()
            .filter(|d| d.reverse_supported)
            .map(|d| d.name)
            .collect();
        assert_eq!(
            rev,
            vec![
                "mysql",
                "postgresql",
                "h2",
                "oracle",
                "dm",
                "kingbase",
                "gbase",
                "sqlserver"
            ]
        );
    }

    #[test]
    fn test_builtin_driver_set() {
        // native 双 + h2(shade) 内置驱动
        let builtin: Vec<_> = ALL_DATABASES
            .iter()
            .filter(|d| d.builtin_driver)
            .map(|d| d.name)
            .collect();
        assert_eq!(builtin, vec!["mysql", "postgresql", "h2"]);
    }
}
